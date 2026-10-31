use ankle_face_protocol::ankleface::Event;
use axum::Router;
use hyper::header::{ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION, CONTENT_TYPE};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    signal::{
        self,
        unix::{signal, SignalKind},
    },
    sync::{self, broadcast},
};
use tower_http::cors::CorsLayer;
use tracing::info;

// rest stuff
mod middleware;
mod routes;
mod utils;

// face stuff
mod grpc;

#[derive(Clone)]
pub struct AppState {
    pub ankle_channel: sync::broadcast::Sender<Event>,
    pub client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    let _guard = backend_common::tracing::init_tracing();

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls cryptoi provider");

    let ankle_channel = broadcast::channel::<Event>(16).0;

    let client = reqwest::Client::new();

    let app_state = Arc::new(AppState {
        ankle_channel,
        client,
    });

    // INFO: GRPC Serve worker
    let grpc_app_state = Arc::clone(&app_state);
    tokio::spawn(async move {
        let app_state = grpc_app_state;
        let host_on_grpc = "[::]:50051";
        let _ = grpc::serve_face(app_state, host_on_grpc).await;
    });

    // INFO: Axum Serve worker
    let axum_app_state = Arc::clone(&app_state);
    tokio::spawn(async move {
        let app_state = axum_app_state;

        let origins = [
            "http://127.0.0.1:5173".parse().unwrap(),
            "https://127.0.0.1:5173".parse().unwrap(),
            "http://10.10.10.10:5173".parse().unwrap(),
            "https://10.10.10.10:5173".parse().unwrap(),
            "https://localhost:5173".parse().unwrap(),
            "https://devinlittle.net".parse().unwrap(),
            "https://api.devinlittle.net".parse().unwrap(),
        ];

        let cors = CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PATCH,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCESS_CONTROL_ALLOW_ORIGIN])
            .allow_credentials(true);

        let app = Router::new().merge(routes::create_routes(app_state).layer(cors));

        let host_on_http = "[::]:3007";

        let handle = axum_server::Handle::new();
        let shutdown_signal_handler = shutdown_signal(handle.clone());

        let listener_tokio = tokio::net::TcpListener::bind(host_on_http).await.unwrap();

        info!(
            "Listening HTTP Server on {}",
            listener_tokio.local_addr().unwrap()
        );
        axum::serve(listener_tokio, app)
            .with_graceful_shutdown(shutdown_signal_handler)
            .await
            .unwrap();
    });

    let _ = signal::ctrl_c().await;
}

async fn shutdown_signal(handle: axum_server::Handle<SocketAddr>) {
    let ctrl_c = signal::ctrl_c();

    let terminte = async {
        signal(SignalKind::terminate())
            .expect("failed to install the SIGTERM handler 🥲")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminte => {},

    }

    info!("Signal recvived now starting graceful shutdown");
    handle.graceful_shutdown(Some(Duration::from_secs(10)));
}

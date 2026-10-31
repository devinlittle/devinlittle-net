use anyhow::Result;
use base64::prelude::*;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{async_trait, transport::Server, Response, Status, Streaming};
use tracing::info;

use crate::grpc::basic_auth::basic_auth_for_grpc;
use crate::AppState;
use ankle_face_protocol::ankleface::{
    ankle_face_server::{AnkleFace, AnkleFaceServer},
    Event,
};

pub mod basic_auth;

type ResponseStream =
    std::pin::Pin<Box<dyn tokio_stream::Stream<Item = std::result::Result<Event, Status>> + Send>>;

pub struct FaceService {
    state: Arc<AppState>,
}

impl FaceService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl AnkleFace for FaceService {
    type ConnectFaceStream = ResponseStream;

    async fn connect_face(
        &self,
        request: tonic::Request<Streaming<Event>>,
    ) -> Result<Response<Self::ConnectFaceStream>, Status> {
        let auth = request
            .metadata()
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;

        let token = auth
            .strip_prefix("Basic ")
            .ok_or_else(|| Status::unauthenticated("Invalid authorization scheme"))?;

        if basic_auth_for_grpc(token).is_err() {
            return Err(Status::unauthenticated("incorrect credentials"));
        }

        let mut inbound_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);
        let output_stream = ReceiverStream::new(rx);

        // INFO: Sender Worker
        let sender_self = Arc::clone(&self.state);
        let sender_grpc_tx = tx.clone();
        tokio::spawn(async move {
            let sender_self = sender_self;
            let sender_grpc_tx = sender_grpc_tx;
            let mut ankle_recv = sender_self.ankle_channel.subscribe();

            //
            loop {
                tokio::select! {
                    msg = ankle_recv.recv() => {
                        match msg {
                            Ok(msg) => {
                                let _ = sender_grpc_tx.send(Ok(msg)).await;
                            },
                            Err(_) => { continue; }
                        }
                    }
                }
            }
            //
        });

        // TODO: have the inbound_stream report live connection to notification (broadcast)
        // > maybe also report battery life/if critical

        Ok(Response::new(
            Box::pin(output_stream) as Self::ConnectFaceStream
        ))
    }
}

pub async fn serve_face(app_state: Arc<AppState>, host_on: &str) -> Result<()> {
    let host_on: SocketAddr = host_on.parse().unwrap();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(ankle_face_protocol::ankleface::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    let face_service = AnkleFaceServer::new(FaceService::new(app_state));

    info!("Listening GRPC Server on {}", host_on.ip());

    Server::builder()
        .add_service(reflection_service)
        .add_service(face_service)
        .serve(host_on)
        .await
        .unwrap();

    Ok(())
}

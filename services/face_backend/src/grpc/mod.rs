use ankle_face_protocol::ankleface::{
    ankle_face_server::{AnkleFace, AnkleFaceServer},
    Event,
};
use anyhow::Result;
use base64::prelude::*;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{async_trait, transport::Server, Response, Status, Streaming};
use tracing::info;

use crate::AppState;
use crate::{grpc::basic_auth::basic_auth_for_grpc, utils::secrets::SECRETS};

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

        println!("remote addr: {:?}", request.remote_addr());

        let mut inbound_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);

        let _ = tx.send(Ok(Event::default())).await;

        let output_stream = ReceiverStream::new(rx);
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

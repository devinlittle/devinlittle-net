use dashmap::DashMap;
use tonic::transport::Channel;
use uuid::Uuid;

use crate::ServiceName;

pub struct MeshConfig {
    pub node_id: Uuid,
    pub friendly_name: String,
    pub service_name: ServiceName,
    pub connected_peers: DashMap<Uuid, PeerConnection>,
}

pub struct PeerConnection {
    pub node_id: Uuid,
    pub friendly_name: String,
    pub service_type: ServiceName,
    pub channel: Channel,
}

pub mod infra {
    tonic::include_proto!("mesh.infra");
}

// TODO: add func to setup interna
// when setting up tonic add cfg feature lines on each service

// begin_service_connection(AppS)

use std::sync::Arc;

use protosocket_prost::ProstSerializer;
use protosocket_rpc::server::SocketService;
use rmemstore_messages::{Response, Rpc};

use crate::{connection_service::RMemstoreConnectionService, rmemstore_server::RMemstoreServer};

pub struct RMemstoreSocketService {
    server: Arc<RMemstoreServer>,
}

impl RMemstoreSocketService {
    pub fn new(server: Arc<RMemstoreServer>) -> Self {
        Self { server }
    }
}

impl SocketService for RMemstoreSocketService {
    type RequestDeserializer = ProstSerializer<Rpc, Response>;
    type ResponseSerializer = ProstSerializer<Rpc, Response>;
    type ConnectionService = RMemstoreConnectionService;

    fn deserializer(&self) -> Self::RequestDeserializer {
        ProstSerializer::default()
    }

    fn serializer(&self) -> Self::ResponseSerializer {
        ProstSerializer::default()
    }

    fn new_connection_service(&self, address: std::net::SocketAddr) -> Self::ConnectionService {
        log::info!("new connection from: {address}");
        RMemstoreConnectionService::new(address, self.server.clone())
    }
}

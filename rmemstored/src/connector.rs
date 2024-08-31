use std::sync::Arc;

use protosocket_prost::ProstServerConnectionBindings;
use protosocket_server::ServerConnector;
use rmemstore_messages::{Response, Rpc};

use crate::{connection::RMemstoreConnection, rmemstore_server::RMemstoreServer};

pub struct RMemstoreConnector {
    server: Arc<RMemstoreServer>,
}

impl RMemstoreConnector {
    pub fn new(server: RMemstoreServer) -> Self {
        Self {
            server: Arc::new(server),
        }
    }
}

impl ServerConnector for RMemstoreConnector {
    type Bindings = ProstServerConnectionBindings<Rpc, Response, RMemstoreConnection>;

    fn serializer(&self) -> <Self::Bindings as protosocket::ConnectionBindings>::Serializer {
        protosocket_prost::ProstSerializer::default()
    }

    fn deserializer(&self) -> <Self::Bindings as protosocket::ConnectionBindings>::Deserializer {
        protosocket_prost::ProstSerializer::default()
    }

    fn new_reactor(
        &self,
        outbound: tokio::sync::mpsc::Sender<
            <<Self::Bindings as protosocket::ConnectionBindings>::Serializer as protosocket::Serializer>::Message,
        >,
    ) -> <Self::Bindings as protosocket::ConnectionBindings>::Reactor {
        RMemstoreConnection::new(self.server.clone(), outbound)
    }
}

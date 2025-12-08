use std::sync::Arc;

use protosocket::{PooledEncoder, StreamWithAddress, TcpSocketListener};
use protosocket_prost::{ProstDecoder, ProstSerializer};
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
    type Codec = (PooledEncoder<ProstSerializer<Response>>, ProstDecoder<Rpc>);
    type ConnectionService = RMemstoreConnectionService;
    type SocketListener = TcpSocketListener;

    fn codec(&self) -> Self::Codec {
        Default::default()
    }

    fn new_stream_service(
        &self,
        stream: &StreamWithAddress<tokio::net::TcpStream>,
    ) -> Self::ConnectionService {
        log::info!("new connection from: {}", stream.address());
        RMemstoreConnectionService::new(stream.address(), self.server.clone())
    }
}

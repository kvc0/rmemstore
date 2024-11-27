use std::{net::SocketAddr, sync::Arc};

use futures::{future::BoxFuture, stream::BoxStream, FutureExt};
use protosocket_rpc::{
    server::{ConnectionService, RpcKind},
    ProtosocketControlCode,
};
use rmemstore_messages::Response;

use crate::{commands::command::Command, rmemstore_server::RMemstoreServer};

pub struct RMemstoreConnectionService {
    address: SocketAddr,
    server: Arc<RMemstoreServer>,
}

impl RMemstoreConnectionService {
    pub fn new(address: SocketAddr, server: Arc<RMemstoreServer>) -> Self {
        Self { address, server }
    }
}

impl ConnectionService for RMemstoreConnectionService {
    type Request = rmemstore_messages::Rpc;
    type Response = rmemstore_messages::Response;
    type UnaryFutureType = BoxFuture<'static, Self::Response>;
    type StreamType = BoxStream<'static, Self::Response>;

    fn new_rpc(
        &mut self,
        initiating_message: Self::Request,
    ) -> protosocket_rpc::server::RpcKind<Self::UnaryFutureType, Self::StreamType> {
        log::debug!("{} received message: {initiating_message:?}", self.address);
        let id = initiating_message.id;
        match initiating_message.command {
            Some(command) => {
                let server = self.server.clone();
                match command {
                    rmemstore_messages::rpc::Command::Put(put) => RpcKind::Unary(
                        async move {
                            Response {
                                id,
                                code: ProtosocketControlCode::Normal.as_u8() as u32,
                                kind: put.run(&server),
                            }
                        }
                        .boxed(),
                    ),
                    rmemstore_messages::rpc::Command::Get(get) => RpcKind::Unary(
                        async move {
                            Response {
                                id,
                                code: ProtosocketControlCode::Normal.as_u8() as u32,
                                kind: get.run(&server),
                            }
                        }
                        .boxed(),
                    ),
                }
            }
            None => {
                log::error!("bad command: {initiating_message:?}");
                RpcKind::Unary(
                    async move {
                        Response {
                            id,
                            code: ProtosocketControlCode::Cancel.as_u8() as u32,
                            kind: None,
                        }
                    }
                    .boxed(),
                )
            }
        }
    }
}

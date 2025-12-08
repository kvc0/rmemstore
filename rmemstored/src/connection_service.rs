use std::{net::SocketAddr, sync::Arc};

use protosocket_rpc::{
    server::{ConnectionService, RpcResponder},
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

    fn new_rpc(
        &mut self,
        initiating_message: Self::Request,
        responder: RpcResponder<'_, Self::Response>,
    ) {
        log::debug!("{} received message: {initiating_message:?}", self.address);
        let id = initiating_message.id;
        match initiating_message.command {
            Some(command) => match command {
                rmemstore_messages::rpc::Command::Put(put) => {
                    responder.immediate(Response {
                        id,
                        code: ProtosocketControlCode::Normal.as_u8() as u32,
                        kind: put.run(&self.server),
                    });
                }
                rmemstore_messages::rpc::Command::Get(get) => {
                    responder.immediate(Response {
                        id,
                        code: ProtosocketControlCode::Normal.as_u8() as u32,
                        kind: get.run(&self.server),
                    });
                }
            },
            None => {
                log::error!("bad command: {initiating_message:?}");
                responder.immediate(Response {
                    id,
                    code: ProtosocketControlCode::Cancel.as_u8() as u32,
                    kind: None,
                });
            }
        }
    }
}

use std::sync::Arc;

use protosocket::MessageReactor;
use rmemstore_messages::{rpc, Response, Rpc};

use crate::{commands::command::Command, rmemstore_server::RMemstoreServer};

pub struct RMemstoreConnection {
    outbound: tokio::sync::mpsc::Sender<Response>,
    server: Arc<RMemstoreServer>,
}

impl RMemstoreConnection {
    pub fn new(
        server: Arc<RMemstoreServer>,
        outbound: tokio::sync::mpsc::Sender<Response>,
    ) -> Self {
        Self { outbound, server }
    }
}

impl MessageReactor for RMemstoreConnection {
    type Inbound = Rpc;

    fn on_inbound_messages(
        &mut self,
        messages: impl IntoIterator<Item = Self::Inbound>,
    ) -> protosocket::ReactorStatus {
        for message in messages.into_iter() {
            let id = message.id;
            let kind = match message.command {
                Some(command) => match command {
                    rpc::Command::Put(put) => put.run(&self.server),
                    rpc::Command::Get(get) => get.run(&self.server),
                },
                None => {
                    log::error!("bad command: {message:?}");
                    None
                }
            };
            if let Err(e) = self.outbound.try_send(Response { id, kind }) {
                log::error!("overrun outbound buffer: {e:?}");
                return protosocket::ReactorStatus::Disconnect;
            }
        }

        protosocket::ReactorStatus::Continue
    }
}

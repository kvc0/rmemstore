use std::sync::Arc;

use protosocket::MessageReactor;
use protosocket_prost::ProstServerConnectionBindings;
use protosocket_server::ServerConnector;
use rmemstore_messages::{response, rpc, Response, Rpc};

use crate::{
    rmemstore_server::RMemstoreServer,
    types::{MemstoreItem, MemstoreValue},
};

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
            let command = match message.command {
                Some(command) => command,
                None => {
                    log::error!("bad command: {message:?}");
                    continue;
                }
            };
            let kind = match command {
                rpc::Command::Put(put) => {
                    let Some(value) = put.value else {
                        log::error!("put with no value command: {id:?}");
                        continue;
                    };
                    let value: MemstoreValue = match value.try_into() {
                        Ok(value) => value,
                        Err(e) => {
                            log::error!("bad value: {e:?}");
                            continue;
                        }
                    };
                    self.server.put(put.key, MemstoreItem::new(value));
                    Some(response::Kind::Ok(true))
                }
                rpc::Command::Get(get) => {
                    let item = self.server.get(&get.key);
                    item.map(MemstoreItem::into_value)
                        .map(|v| response::Kind::Value(v.into()))
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

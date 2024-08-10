use bytes::Buf;
use messages::rmemstore;
use protosocket_prost::ClientRegistry;
use tokio::sync::mpsc;

use crate::{
    conversions::{IntoKey, IntoValue},
    message_reactor::{RMemstoreMessageReactor, RpcRegistrar}, Error,
};

pub struct ClientConfiguration {
    registry: ClientRegistry,
}

impl ClientConfiguration {
    pub fn new() -> Self {
        let registry = protosocket_prost::ClientRegistry::new(tokio::runtime::Handle::current());
        Self { registry }
    }

    pub async fn connect(
        &mut self,
        socket_address: impl Into<String>,
    ) -> Result<Client, crate::Error> {
        let (registrar, reactor) = RMemstoreMessageReactor::new();
        let outbound = self
            .registry
            .register_client::<rmemstore::Rpc, rmemstore::Response, RMemstoreMessageReactor>(
                socket_address,
                reactor,
            )
            .await?;

        Ok(Client::new(registrar, outbound))
    }
}

#[derive(Debug)]
pub struct Client {
    registrar: RpcRegistrar,
    outbound: mpsc::Sender<rmemstore::Rpc>,
}

impl Client {
    pub fn new(registrar: RpcRegistrar, outbound: mpsc::Sender<rmemstore::Rpc>) -> Self {
        Self {
            registrar,
            outbound,
        }
    }

    async fn send_command(
        &self,
        command: rmemstore::rpc::Command,
    ) -> Result<rmemstore::Response, crate::Error> {
        let (id, completion) = self.registrar.preregister_command();
        self.outbound
            .send(rmemstore::Rpc {
                id: id,
                command: Some(command),
            })
            .await
            .map_err(|_| crate::Error::ConnectionBroken("can't send to outbound stream"))?;
        completion.await.map_err(|_| {
            crate::Error::ConnectionBroken("can't receive response - sender was dropped")
        })
    }

    pub async fn put(&self, key: impl IntoKey, value: impl IntoValue) -> Result<(), crate::Error> {
        let command = rmemstore::rpc::Command::Put(rmemstore::Put {
            key: key.into_key(),
            value: Some(rmemstore::Value {
                kind: Some(value.into_value()),
            }),
        });
        self.send_command(command).await?;
        Ok(())
    }

    pub async fn get(&self, key: impl IntoKey) -> Result<(), crate::Error> {
        let command = rmemstore::rpc::Command::Get(rmemstore::Get {
            key: key.into_key(),
        });
        let response = self.send_command(command).await?;
        let Some(response) = response.kind else {
            return Err(Error::MalformedResponse("missing response kind"))
        };
        match response {
            rmemstore::response::Kind::Value(v) => {
                let Some(value) = v.kind else {
                    return Err(Error::MalformedResponse("missing value kind"))
                };
                match value {
                    rmemstore::value::Kind::Blob(v) => {
                        match std::io::read_to_string(v.reader()) {
                            Ok(s) => {
                                println!("{s}");
                            }
                            Err(e) => {
                                println!("unsupported value: {e:?}");
                            }
                        }
                    }
                    rmemstore::value::Kind::Map(map) => {
                        println!("{:#?}", map.map);
                    }
                }
            }
            _ => {
                return Err(Error::MalformedResponse("incorrect response type"));
            }
        }
        Ok(())
    }
}

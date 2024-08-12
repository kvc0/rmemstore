use protosocket_prost::ClientRegistry;
use tokio::sync::mpsc;

use crate::{
    message_reactor::{RMemstoreMessageReactor, RpcRegistrar},
    types::{IntoKey, IntoValue, MemstoreValue},
    Error,
};

pub struct ClientConfiguration {
    registry: ClientRegistry,
}

impl ClientConfiguration {
    #[allow(clippy::new_without_default)]
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
            .register_client::<rmemstore_messages::Rpc, rmemstore_messages::Response, RMemstoreMessageReactor>(
                socket_address,
                reactor,
            )
            .await?;

        Ok(Client::new(registrar, outbound))
    }
}

/// Cheap to clone, this is how you call rmemstored.
#[derive(Debug, Clone)]
pub struct Client {
    registrar: RpcRegistrar,
    outbound: mpsc::Sender<rmemstore_messages::Rpc>,
}

impl Client {
    pub fn new(registrar: RpcRegistrar, outbound: mpsc::Sender<rmemstore_messages::Rpc>) -> Self {
        Self {
            registrar,
            outbound,
        }
    }

    async fn send_command(
        &self,
        command: rmemstore_messages::rpc::Command,
    ) -> Result<rmemstore_messages::Response, crate::Error> {
        let (id, completion) = self.registrar.preregister_command();
        self.outbound
            .send(rmemstore_messages::Rpc {
                id,
                command: Some(command),
            })
            .await
            .map_err(|_| crate::Error::ConnectionBroken("can't send to outbound stream"))?;
        completion.await.map_err(|_| {
            crate::Error::ConnectionBroken("can't receive response - sender was dropped")
        })
    }

    pub async fn put(&self, key: impl IntoKey, value: impl IntoValue) -> Result<(), crate::Error> {
        let command = rmemstore_messages::rpc::Command::Put(rmemstore_messages::Put {
            key: key.into_key(),
            value: Some(rmemstore_messages::Value {
                kind: Some(value.into_value()),
            }),
        });
        self.send_command(command).await?;
        Ok(())
    }

    pub async fn get(&self, key: impl IntoKey) -> Result<MemstoreValue, crate::Error> {
        let command = rmemstore_messages::rpc::Command::Get(rmemstore_messages::Get {
            key: key.into_key(),
        });
        let response = self.send_command(command).await?;
        let Some(response) = response.kind else {
            return Err(Error::MalformedResponse("missing response kind"));
        };
        match response {
            rmemstore_messages::response::Kind::Value(value) => value.try_into(),
            _ => Err(Error::MalformedResponse("incorrect response type")),
        }
    }
}

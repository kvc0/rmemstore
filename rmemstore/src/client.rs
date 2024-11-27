use std::{
    net::SocketAddr,
    sync::{atomic::AtomicU64, Arc},
};

use protosocket_prost::ProstSerializer;
use protosocket_rpc::{client::Configuration, ProtosocketControlCode};
use rmemstore_messages::{response, Response, Rpc};

use crate::{
    types::{IntoKey, IntoValue, MemstoreValue},
    Error,
};

/// Cheap to clone, this is how you call rmemstored.
#[derive(Debug, Clone)]
pub struct Client {
    client: protosocket_rpc::client::RpcClient<Rpc, Response>,
    command_id: Arc<AtomicU64>,
}

#[derive(Debug, Clone)]
pub struct ConnectionConfiguration {
    max_message_size: usize,
    queued_messages: usize,
}

impl Default for ConnectionConfiguration {
    fn default() -> Self {
        Self {
            max_message_size: 4 * (2 << 20),
            queued_messages: 256,
        }
    }
}

impl ConnectionConfiguration {
    pub fn max_message_size(&mut self, max_message_size: usize) {
        self.max_message_size = max_message_size;
    }

    pub fn queued_messages(&mut self, queued_messages: usize) {
        self.queued_messages = queued_messages;
    }
}

impl Client {
    pub async fn connect(
        address: SocketAddr,
        configuration: ConnectionConfiguration,
    ) -> Result<Self, crate::Error> {
        let mut client_configuration = Configuration::default();
        client_configuration.max_buffer_length(configuration.max_message_size);
        client_configuration.max_queued_outbound_messages(configuration.queued_messages);
        let (client, connection_driver) = protosocket_rpc::client::connect::<
            ProstSerializer<Response, Rpc>,
            ProstSerializer<Response, Rpc>,
        >(address, &client_configuration)
        .await?;
        tokio::spawn(connection_driver);
        Ok(Self {
            client,
            command_id: Arc::new(AtomicU64::new(1)),
        })
    }

    async fn send_command(
        &self,
        command: rmemstore_messages::rpc::Command,
    ) -> Result<rmemstore_messages::Response, crate::Error> {
        let id = self
            .command_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(self
            .client
            .send_unary(rmemstore_messages::Rpc {
                id,
                code: ProtosocketControlCode::Normal.as_u8() as u32,
                command: Some(command),
            })
            .await?
            .await?)
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

    pub async fn get(&self, key: impl IntoKey) -> Result<Option<MemstoreValue>, crate::Error> {
        let command = rmemstore_messages::rpc::Command::Get(rmemstore_messages::Get {
            key: key.into_key(),
        });
        let response = self.send_command(command).await?;
        match response.kind {
            Some(response::Kind::Value(value)) => Ok(Some(value.try_into()?)),
            Some(other) => {
                log::debug!("unexpected response: {other:?}");
                Err(Error::MalformedResponse("incorrect response type"))
            }
            _ => Ok(None),
        }
    }
}

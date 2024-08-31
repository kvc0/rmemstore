use rmemstore_messages::response;

use crate::{
    rmemstore_server::RMemstoreServer,
    types::{MemstoreItem, MemstoreValue},
};

use super::command::Command;

impl Command for rmemstore_messages::Put {
    fn run(self, server: &RMemstoreServer) -> Option<rmemstore_messages::response::Kind> {
        let Some(value) = self.value else {
            log::error!("put with no value");
            return None;
        };
        let value: MemstoreValue = match value.try_into() {
            Ok(value) => value,
            Err(e) => {
                log::error!("bad value: {e:?}");
                return None;
            }
        };
        server.put(self.key, MemstoreItem::new(value));
        Some(response::Kind::Ok(true))
    }
}

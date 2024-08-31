use rmemstore_messages::response;

use crate::{rmemstore_server::RMemstoreServer, types::MemstoreItem};

use super::command::Command;

impl Command for rmemstore_messages::Get {
    fn run(self, server: &RMemstoreServer) -> Option<rmemstore_messages::response::Kind> {
        let item = server.get(&self.key);
        item.map(MemstoreItem::into_value)
            .map(|v| response::Kind::Value(v.into()))
    }
}

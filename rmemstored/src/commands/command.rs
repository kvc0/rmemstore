use crate::rmemstore_server::RMemstoreServer;

pub trait Command {
    fn run(self, server: &RMemstoreServer) -> Option<rmemstore_messages::response::Kind>;
}

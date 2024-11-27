use crate::{Response, Rpc};

impl protosocket_rpc::Message for Rpc {
    fn message_id(&self) -> u64 {
        self.id
    }

    fn control_code(&self) -> protosocket_rpc::ProtosocketControlCode {
        protosocket_rpc::ProtosocketControlCode::from_u8(self.code as u8)
    }

    fn set_message_id(&mut self, message_id: u64) {
        self.id = message_id;
    }

    fn cancelled(message_id: u64) -> Self {
        Self {
            id: message_id,
            code: protosocket_rpc::ProtosocketControlCode::Cancel.as_u8() as u32,
            command: None,
        }
    }

    fn ended(message_id: u64) -> Self {
        Self {
            id: message_id,
            code: protosocket_rpc::ProtosocketControlCode::End.as_u8() as u32,
            command: None,
        }
    }
}

impl protosocket_rpc::Message for Response {
    fn message_id(&self) -> u64 {
        self.id
    }

    fn control_code(&self) -> protosocket_rpc::ProtosocketControlCode {
        protosocket_rpc::ProtosocketControlCode::from_u8(self.code as u8)
    }

    fn set_message_id(&mut self, message_id: u64) {
        self.id = message_id;
    }

    fn cancelled(message_id: u64) -> Self {
        Self {
            id: message_id,
            code: protosocket_rpc::ProtosocketControlCode::Cancel.as_u8() as u32,
            kind: None,
        }
    }

    fn ended(message_id: u64) -> Self {
        Self {
            id: message_id,
            code: protosocket_rpc::ProtosocketControlCode::End.as_u8() as u32,
            kind: None,
        }
    }
}

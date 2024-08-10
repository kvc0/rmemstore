use std::{
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc},
};

use messages::rmemstore;
use protosocket::MessageReactor;
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct SubmittedRpc {
    pub id: u64,
    pub completion: tokio::sync::oneshot::Sender<rmemstore::Response>,
}

#[derive(Debug, Clone)]
pub struct RpcRegistrar {
    in_flight_submission: Arc<k_lock::Mutex<Vec<SubmittedRpc>>>,
    message_id: Arc<AtomicU64>,
}

impl RpcRegistrar {
    /// Get a linked command id for which you will then send a command
    pub fn preregister_command(&self) -> (u64, oneshot::Receiver<rmemstore::Response>) {
        let id = self
            .message_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (completion, response) = oneshot::channel();
        self.in_flight_submission
            .lock()
            .expect("mutex must not be poisoned")
            .push(SubmittedRpc { id, completion });
        (id, response)
    }
}

#[derive(Debug)]
pub struct RMemstoreMessageReactor {
    in_flight_submission: Arc<k_lock::Mutex<Vec<SubmittedRpc>>>,
    in_flight_buffer: Vec<SubmittedRpc>,
    in_flight: HashMap<u64, tokio::sync::oneshot::Sender<rmemstore::Response>>,
}

impl RMemstoreMessageReactor {
    pub(crate) fn new() -> (RpcRegistrar, Self) {
        let in_flight_submission = Arc::new(k_lock::Mutex::new(Vec::new()));
        (
            RpcRegistrar {
                in_flight_submission: in_flight_submission.clone(),
                message_id: Arc::new(AtomicU64::new(1)),
            },
            Self {
                in_flight_submission,
                in_flight_buffer: Default::default(),
                in_flight: Default::default(),
            },
        )
    }
}

impl MessageReactor for RMemstoreMessageReactor {
    type Inbound = rmemstore::Response;

    fn on_inbound_messages(
        &mut self,
        messages: impl IntoIterator<Item = Self::Inbound>,
    ) -> protosocket::ReactorStatus {
        // Atomic O(1) swap of the submission queue to keep the mutex as brief as possible
        std::mem::swap(
            &mut self.in_flight_buffer,
            &mut *self
                .in_flight_submission
                .lock()
                .expect("mutex must not be poisoned"),
        );
        // register the newly arrived commands outside of the lock
        self.in_flight.extend(
            self.in_flight_buffer
                .drain(..)
                .map(|SubmittedRpc { id, completion }| (id, completion)),
        );
        // respond to messages
        for response in messages.into_iter() {
            match self.in_flight.remove(&response.id) {
                Some(completion) => {
                    let _ = completion.send(response);
                }
                None => {
                    log::error!("received response for unregistered command {response:?}");
                }
            }
        }
        protosocket::ReactorStatus::Continue
    }
}

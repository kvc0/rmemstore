use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use bytes::Bytes;
use protosocket::{MessageReactor, ReactorStatus};
use rmemstore_messages::{rpc, Response, Rpc};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let threads = 2;
    let client_tasks = 4;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name_fn(|| {
            static I: AtomicUsize = AtomicUsize::new(0);
            let i = I.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("app-{i}")
        })
        .worker_threads(threads)
        .enable_all()
        .build()?;

    runtime.block_on(run_main(client_tasks))
}

async fn run_main(uploaders: usize) -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut registry = protosocket_prost::ClientRegistry::new(tokio::runtime::Handle::current());
    registry.set_max_read_buffer_length(16 * 1024);

    let response_count = Arc::new(AtomicUsize::new(0));
    let latency = Arc::new(histogram::AtomicHistogram::new(7, 32).expect("histogram works"));

    let mut uploader_tasks = tokio::task::JoinSet::new();
    for _i in 0..uploaders {
        let concurrent_count = Arc::new(Semaphore::new(128));
        let concurrent = Arc::new(Mutex::new(HashMap::with_capacity(
            concurrent_count.available_permits(),
        )));
        let outbound = registry
            .register_client::<Rpc, Response, ProtoCompletionReactor>(
                std::env::var("ENDPOINT").unwrap_or_else(|_| "127.0.0.1:9001".to_string()),
                ProtoCompletionReactor {
                    count: response_count.clone(),
                    latency: latency.clone(),
                    concurrent: concurrent.clone(),
                    concurrent_wip: Default::default(),
                },
            )
            .await?;
        uploader_tasks.spawn(run_message_generator(
            concurrent_count,
            concurrent.clone(),
            outbound,
        ));
    }

    let metrics = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            let start = Instant::now();
            interval.tick().await;
            let total = response_count.swap(0, std::sync::atomic::Ordering::Relaxed);
            let hz = (total as f64) / start.elapsed().as_secs_f64().max(0.1);

            let latency = latency.drain();
            let p90 = *latency
                .percentile(0.9)
                .unwrap_or_default()
                .expect("come on")
                .range()
                .end() as f64
                / 1000.0;
            let p999 = *latency
                .percentile(0.999)
                .unwrap_or_default()
                .expect("come on")
                .range()
                .end() as f64
                / 1000.0;
            let p9999 = *latency
                .percentile(0.9999)
                .unwrap_or_default()
                .expect("come on")
                .range()
                .end() as f64
                / 1000.0;
            eprintln!("Messages: {total:10} rate: {hz:9.1}hz p90: {p90:6.1}µs p999: {p999:6.1}µs p9999: {p9999:6.1}µs");
        }
    });

    tokio::select!(
        _ = uploader_tasks.join_next() => {
            log::warn!("uploader quit");
        }
        _ = metrics => {
            log::warn!("metrics runtime quit");
        }
    );

    Ok(())
}

struct ProtoCompletionReactor {
    count: Arc<AtomicUsize>,
    latency: Arc<histogram::AtomicHistogram>,
    concurrent: Arc<Mutex<HashMap<u64, (OwnedSemaphorePermit, u128)>>>,
    concurrent_wip: HashMap<u64, (OwnedSemaphorePermit, u128)>,
}
impl MessageReactor for ProtoCompletionReactor {
    type Inbound = Response;

    fn on_inbound_messages(
        &mut self,
        messages: impl IntoIterator<Item = Self::Inbound>,
    ) -> ReactorStatus {
        log::debug!("received message response batch");
        // make sure you hold the concurrent lock as briefly as possible.
        // the permits will be released and the threads will race to lock
        // otherwise. This could also be triple-buffered so the lock is
        // only the duration of a pointer swap.
        self.concurrent_wip
            .extend(self.concurrent.lock().expect("mutex works").drain());
        for response in messages.into_iter() {
            let (concurrency_permit, timestamp) = self
                .concurrent_wip
                .remove(&response.id)
                .expect("must not receive messages that have not been sent");
            drop(concurrency_permit);
            let latency = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time works")
                .as_nanos()
                - timestamp;
            let _ = self.latency.increment(latency as u64);
            assert_ne!(response.id, 0, "received bad message");
            self.count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        ReactorStatus::Continue
    }
}

async fn run_message_generator(
    concurrent_count: Arc<Semaphore>,
    concurrent: Arc<Mutex<HashMap<u64, (OwnedSemaphorePermit, u128)>>>,
    outbound: tokio::sync::mpsc::Sender<Rpc>,
) {
    log::debug!("running producer");
    let mut i = 1;
    loop {
        let permit = concurrent_count
            .clone()
            .acquire_owned()
            .await
            .expect("semaphore works");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time works")
            .as_nanos();
        concurrent
            .lock()
            .expect("mutex works")
            .insert(i, (permit, now));
        match outbound
            .send(Rpc {
                id: i,
                command: Some(command(i)),
            })
            .await
        {
            Ok(_) => {
                i += 1;
            }
            Err(e) => {
                log::error!("send should work: {e:?}");
                return;
            }
        }
    }
}

fn command(i: u64) -> rpc::Command {
    let item = i % (2 << 21);
    match i % 1000 {
        0..100 => rpc::Command::Put(rmemstore_messages::Put {
            key: Bytes::copy_from_slice(&item.to_be_bytes()),
            value: Some(rmemstore_messages::Value {
                kind: Some(rmemstore_messages::value::Kind::Blob(
                    Bytes::copy_from_slice(&(item + 1).to_be_bytes()),
                )),
            }),
        }),
        100..1000 => rpc::Command::Get(rmemstore_messages::Get {
            key: Bytes::copy_from_slice(&item.to_be_bytes()),
        }),
        _ => rpc::Command::Get(rmemstore_messages::Get {
            key: Bytes::copy_from_slice(&item.to_be_bytes()),
        }),
    }
}

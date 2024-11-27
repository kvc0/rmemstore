use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::{Duration, Instant},
};

use bytes::Bytes;
use rmemstore::ConnectionConfiguration;
use tokio::task::JoinSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let threads = 4;
    let connections = 4;
    let concurrency_per_connection = 64;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name_fn(|| {
            static I: AtomicUsize = AtomicUsize::new(0);
            let i = I.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("app-{i}")
        })
        .worker_threads(threads)
        .enable_all()
        .build()?;

    runtime.block_on(run_main(connections, concurrency_per_connection))
}

async fn run_main(
    connections: usize,
    concurrency_per_connection: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let response_count = Arc::new(AtomicUsize::new(0));
    let latency = Arc::new(histogram::AtomicHistogram::new(7, 32).expect("histogram works"));
    let mut configuration = ConnectionConfiguration::default();
    configuration.queued_messages(256);
    configuration.max_message_size(32 * (1 << 20));

    let mut uploader_tasks = tokio::task::JoinSet::new();
    for _i in 0..connections {
        let client = rmemstore::Client::connect(
            std::env::var("ENDPOINT")
                .unwrap_or_else(|_| "127.0.0.1:9466".to_string())
                .parse()
                .expect("ENDPOINT must be a socket address"),
            configuration.clone(),
        )
        .await?;
        uploader_tasks.spawn(run_message_generator_for_connection(
            concurrency_per_connection,
            client,
            response_count.clone(),
            latency.clone(),
        ));
    }

    let metrics = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            let start = Instant::now();
            interval.tick().await;
            let total = response_count.swap(0, std::sync::atomic::Ordering::Relaxed);
            // let total_megabytes = throughput_bytes.swap(0, std::sync::atomic::Ordering::Relaxed)
            //     as f64
            //     / 1024.0
            //     / 1024.0;
            let hz = (total as f64) / start.elapsed().as_secs_f64().max(0.1);

            let latency = latency.drain();
            let p90 = *latency
                .percentile(0.9)
                .unwrap_or_default()
                .map(|b| b.range())
                .unwrap_or(0..=0)
                .end() as f64
                / 1000.0;
            let p999 = *latency
                .percentile(0.999)
                .unwrap_or_default()
                .map(|b| b.range())
                .unwrap_or(0..=0)
                .end() as f64
                / 1000.0;
            let p9999 = *latency
                .percentile(0.9999)
                .unwrap_or_default()
                .map(|b| b.range())
                .unwrap_or(0..=0)
                .end() as f64
                / 1000.0;
            // let megabytes_rate = total_megabytes / interval.period().as_secs_f64();
            eprintln!("Messages: {total:10} rate: {hz:9.1}hz, p90: {p90:6.1}µs p999: {p999:6.1}µs p9999: {p9999:6.1}µs");
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

async fn run_message_generator_for_connection(
    concurrent_count: usize,
    client: rmemstore::Client,
    count: Arc<AtomicUsize>,
    latency: Arc<histogram::AtomicHistogram>,
) {
    log::debug!("running producer");
    let mut tasks: JoinSet<_> = (0..concurrent_count)
        .map(|_k| {
            let client = client.clone();
            let count = count.clone();
            let latency = latency.clone();
            tokio::spawn(async move {
                static ITEM_COUNT: u64 = 1000;
                static PAYLOAD: std::sync::LazyLock<Bytes> =
                    std::sync::LazyLock::new(|| Bytes::from_iter((0..1024).map(|i| i as u8)));

                let mut i: u64 = 0;
                loop {
                    i += 1;

                    let now = Instant::now();

                    let die: f32 = rand::random();
                    let item = i % ITEM_COUNT;
                    match die {
                        0.0..0.5 => {
                            client
                                .put(
                                    Bytes::copy_from_slice(&item.to_be_bytes()),
                                    rmemstore::types::MemstoreValue::Blob {
                                        value: PAYLOAD.clone(),
                                    },
                                )
                                .await
                                .expect("it should work");
                        }
                        0.5..1.0 => {
                            client
                                .get(Bytes::copy_from_slice(&item.to_be_bytes()))
                                .await
                                .expect("it should work");
                        }
                        _ => {
                            client
                                .get(Bytes::copy_from_slice(&item.to_be_bytes()))
                                .await
                                .expect("it should work");
                        }
                    };
                    latency
                        .increment(now.elapsed().as_nanos() as u64)
                        .expect("hdr histogram is as it is");
                    count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            })
        })
        .collect();
    while let Some(next) = tasks.join_next().await {
        log::warn!("task quit: {next:?}");
    }
}

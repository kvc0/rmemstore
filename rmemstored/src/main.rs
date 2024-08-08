use std::sync::atomic::AtomicUsize;

use clap::Parser;
use rmemstore_server::RMemstoreServer;

mod connector;
mod options;
mod rmemstore_server;
mod types;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    let options = options::Options::parse();
    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or(&options.log_level)
            .default_write_style_or("always"),
    )
    .init();

    let connection_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .event_interval(3)
        .worker_threads(options.worker_threads)
        .thread_name_fn(|| {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            format!("conn-{}", id)
        })
        .build()
        .expect("must be able to build worker runtime");

    let server = RMemstoreServer::new();

    match options.run_mode {
        options::ServerMode::Plaintext { socket_address } => {
            let server = connection_runtime
                .block_on(protosocket_server::ProtosocketServer::new(
                    socket_address,
                    connection_runtime.handle().clone(),
                    connector::RMemstoreConnector::new(server),
                ))
                .expect("can create a server");
            log::info!("serving on {socket_address}");
            connection_runtime.block_on(server);
        }
    }
}

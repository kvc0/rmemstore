use std::sync::atomic::AtomicUsize;

use clap::Parser;
use rmemstore_server::RMemstoreServer;

mod commands;
mod connection;
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
    log::info!("{options:?}");

    let worker_threads = if options.worker_threads == 0 {
        num_cpus::get_physical().saturating_sub(1).max(1)
    } else {
        options.worker_threads
    };
    let segments = (worker_threads as f64 * 1.5).ceil() as usize;

    let connection_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .event_interval(3)
        .worker_threads(worker_threads)
        .thread_name_fn(|| {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            format!("conn-{}", id)
        })
        .build()
        .expect("must be able to build worker runtime");

    let server = RMemstoreServer::new(segments, options.cache_bytes);

    let signals = signals::Signals::register().expect("must be able to register signals");

    match options.run_mode {
        options::ServerMode::Plaintext { socket_address } => {
            let mut server = connection_runtime
                .block_on(protosocket_server::ProtosocketServer::new(
                    socket_address,
                    connection_runtime.handle().clone(),
                    connector::RMemstoreConnector::new(server),
                ))
                .expect("can create a server");
            server.set_max_buffer_length(options.request_buffer_bytes);
            log::info!("serving on {socket_address}");
            let join_handle = connection_runtime.spawn(server);
            connection_runtime.block_on(async move {
                tokio::select! {
                    _ = signals.wait_for_termination() => {
                        log::warn!("terminal signal");
                    }
                    _ = join_handle => {
                        log::warn!("server exited");
                    }
                }
            })
        }
    }
}

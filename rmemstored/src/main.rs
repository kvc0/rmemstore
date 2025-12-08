use std::sync::Arc;

use clap::Parser;
use protosocket::TcpSocketListener;
use protosocket_rpc::server::LevelSpawn;
use rmemstore_server::RMemstoreServer;

mod commands;
mod connection_service;
mod options;
mod rmemstore_server;
mod socket_service;
mod types;

use socket_service::RMemstoreSocketService;
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;
use tokio::task::spawn_blocking;

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

    let runtime = level_runtime::Builder::default()
        .worker_threads(options.worker_threads)
        .enable_all()
        .thread_name_prefix("conn")
        .event_interval(3)
        .build();

    let server = Arc::new(RMemstoreServer::new(segments, options.cache_bytes));

    let signals = signals::Signals::register().expect("must be able to register signals");

    match options.run_mode {
        options::ServerMode::Plaintext { socket_address } => {
            runtime.handle().spawn_on_each(move || {
                let server = server.clone();
                async move {
                    let mut server = protosocket_rpc::server::SocketRpcServer::new_with_spawner(
                        TcpSocketListener::listen(socket_address, 4, None)?,
                        RMemstoreSocketService::new(server.clone()),
                        4 << 20,
                        1 << 20,
                        128,
                        LevelSpawn::default(),
                    )
                    .await
                    .expect("must be able to listen");
                    server.set_max_queued_outbound_messages(512);
                    server.set_max_buffer_length(options.request_buffer_bytes);
                    server.await
                }
            });

            log::info!("serving on {socket_address}");
            tokio::runtime::Builder::new_current_thread()
                .build()
                .expect("runtime")
                .block_on(async move {
                    let join_handle = Arc::new(spawn_blocking(move || runtime.run()));

                    tokio::select! {
                        _ = signals.wait_for_termination() => {
                            log::warn!("terminal signal");
                        }
                        // fixme: need to make levelruntime stoppable
                    }
                })
        }
    }
}

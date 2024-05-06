use std::{error::Error, io::{self, Read}, time::{Duration, Instant}};

use clap::Parser;
use connection_handler::{ConnectionAllocator, ConnectionHandler};
use mio::{net::TcpListener, Events, Interest, Poll, Token};
use rand::{distributions::Distribution, Rng, RngCore};

mod connection_handler;
mod options;

fn main() {
    let options = options::Options::parse();
    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or(&options.log_level)
            .default_write_style_or("always"),
    )
    .init();

    let poll = Poll::new().expect("must be able to create a poll");
    // Events is collection of readiness Events
    let events = Events::with_capacity(64);

    let connection_handlers: Vec<ConnectionAllocator> = (0..options.worker_threads).map(|i| ConnectionHandler::new_thread(format!("h-{i:02}"))).collect();

    match options.run_mode {
        options::ServerMode::Plaintext { socket_address } => {
            let mut listener = TcpListener::bind(socket_address).expect("must be able to bind to configured address");
            poll.registry().register(&mut listener, SERVER, Interest::READABLE).expect("must be able to register listener");
            serve(poll, events, listener, connection_handlers).expect("server must complete")
        }
    }
}

const SERVER: Token = Token(0);

fn serve(mut poll: Poll, mut events: Events, listener: TcpListener, connection_handlers: Vec<ConnectionAllocator>) -> Result<(), Box<dyn Error>> {
    let mut rng = rand::thread_rng();
    let mut last_balance: Instant = Instant::now();
    let mut connection_distribution = rand::distributions::WeightedIndex::new(connection_handlers.iter().map(|c| c.weight())).expect("failed to make weights");
    loop {
        // Poll the OS for events, waiting at most 100 milliseconds.
        poll.poll(&mut events, Some(Duration::from_millis(100)))?;

        // Process each event.
        for event in events.iter() {
            // We can use the token we previously provided to `register` to
            // determine for which type the event is.
            match event.token() {
                SERVER => loop {
                    // One or more connections are ready, so we'll attempt to
                    // accept them (in a loop).
                    match listener.accept() {
                        Ok((connection, address)) => {
                            log::info!("Got a connection from: {}", address);
                            if Duration::from_millis(200) < last_balance.elapsed() {
                                connection_distribution = rand::distributions::WeightedIndex::new(connection_handlers.iter().map(|c| c.weight())).expect("failed to make weights");
                                last_balance = Instant::now();
                            }
                            let index = connection_distribution.sample(&mut rng);
                            connection_handlers[index].allocate(connection);
                        },
                        // A "would block error" is returned if the operation
                        // is not ready, so we'll stop trying to accept
                        // connections.
                        Err(ref err) if would_block(err) => break,
                        Err(err) => return Err(err.into()),
                    }
                }
                t => {
                    panic!("unhandled token {t:?}")
                }
            }
        }
    }
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

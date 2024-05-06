use std::{collections::HashMap, sync::mpsc::{self, Receiver}, thread::JoinHandle, time::Duration};

use mio::{event::Event, net::TcpStream, Events, Interest, Poll, Token};

pub struct ConnectionHandler {
    connection_receiver: mpsc::Receiver<TcpStream>,
    poll: Poll,
    events: Events,
    connections: slab::Slab<Connection>,
}

impl ConnectionHandler {
    pub fn new_thread(name: String) -> ConnectionAllocator {
        let (sender, connection_receiver) = mpsc::channel();
        let handler = Self {
            connection_receiver,
            poll: Poll::new().expect("must be able to make a poll"),
            events: Events::with_capacity(16),
            connections: slab::Slab::with_capacity(16),
        };
        let handle = std::thread::Builder::new().name(name).spawn(move || {
            handler.run()
        }).expect("must be able to spawn handler");
        ConnectionAllocator::new(sender, handle)
    }

    fn run(mut self) {
        loop {
            while let Ok(new_connection) = self.connection_receiver.try_recv() {
                let entry = self.connections.vacant_entry();
                let token = Token(entry.key());
                let new_connection = entry.insert(Connection::new(new_connection));

                if let Err(e) = new_connection.register(&self.poll, token) {
                    log::error!("failed to register connection {e:?}");
                }
            }
            self.poll.poll(&mut self.events, Some(Duration::from_millis(10))).expect("poll must work");
            for event in &self.events {
                match self.connections.get(event.token().0) {
                    Some(connection) => {
                        match connection.handle(event) {
                            ConnectionState::Ok => {
                                log::trace!("handled event")
                            }
                            ConnectionState::Failed => {
                                let connection = self.connections.remove(event.token().0);
                                if let Err(e) = connection.deregister(&self.poll) {
                                    log::error!("could not deregister connection: {e:?}");
                                }
                            }
                        }
                    }
                    None => {
                        log::error!("received event for nonexistent connection");
                    }
                }
            }
        }
    }
}

pub struct ConnectionAllocator {
    sender: mpsc::Sender<TcpStream>,
    _handler: JoinHandle<()>,
}

impl ConnectionAllocator {
    pub fn new(sender: mpsc::Sender<TcpStream>, handler: JoinHandle<()>) -> Self {
        Self { sender, _handler: handler }
    }

    pub fn weight(&self) -> i64 {
        1
    }

    pub fn allocate(&self, connection: TcpStream) {
        self.sender.send(connection).expect("connection handlers do not hang up")
    }
}

struct Connection {
    connection: TcpStream,
}

impl Connection {
    pub fn new(connection: TcpStream) -> Self {
        Self { connection }
    }

    pub fn register(&mut self, poll: &Poll, token: Token) -> Result<(), std::io::Error> {
        poll.registry().register(&mut self.connection, token, Interest::READABLE | Interest::WRITABLE)
    }

    pub fn deregister(mut self, poll: &Poll) -> Result<(), std::io::Error> {
        poll.registry().deregister(&mut self.connection)
    }

    #[must_use]
    pub fn handle(&self, event: &Event) -> ConnectionState {
        // event.is_error()
        ConnectionState::Ok
    }
}

enum ConnectionState {
    Ok,
    Failed,
}

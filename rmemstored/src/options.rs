use std::{io, net::SocketAddr};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(about = "Memstored service")]
pub struct Options {
    #[arg(long, default_value = "info")]
    pub log_level: String,

    #[arg(long, default_value = "1")]
    pub worker_threads: usize,

    #[command(subcommand)]
    pub run_mode: ServerMode,
}

#[derive(Subcommand)]
pub enum ServerMode {
    Plaintext {
        #[arg(help = "Tcp listen port", default_value = "0.0.0.0:9001", value_parser = parse_address)]
        socket_address: SocketAddr,
    },
    // Tls {
    //     #[arg(
    //         long,
    //         help = "private key pem file",
    //     )]
    //     private_key: String,
    //     #[arg(
    //         long,
    //         help = "public key pem file",
    //     )]
    //     public_key: String,
    // },
    // SelfSigned {
    //     #[arg(help = "hostname to serve as")]
    //     hostname: String,
    // },
}

fn parse_address(arg: &str) -> io::Result<SocketAddr> {
    std::net::ToSocketAddrs::to_socket_addrs(arg)?
        .next()
        .ok_or(io::Error::new(
            io::ErrorKind::Other,
            "must pass a valid socket address",
        ))
}

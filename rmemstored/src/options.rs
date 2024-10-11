use std::{io, net::SocketAddr};

use clap::{Parser, Subcommand};

#[derive(Parser, Clone, Debug)]
#[clap(about = "Memstored service")]
pub struct Options {
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// 0 means n_cpus - 1
    #[arg(long, default_value = "0")]
    pub worker_threads: usize,

    /// 0 means n_cpus - 1
    #[arg(long = "size", default_value = "1gib", value_parser=parse_bytes)]
    pub cache_bytes: usize,

    /// max buffer size
    #[arg(long = "buffer", default_value = "128mib", value_parser=parse_bytes)]
    pub request_buffer_bytes: usize,

    #[command(subcommand)]
    pub run_mode: ServerMode,
}

fn parse_bytes(s: &str) -> Result<usize, clap::Error> {
    parse_size::parse_size(s).map(|n| n as usize).map_err(|e| {
        log::error!("{e:?}");
        clap::Error::new(clap::error::ErrorKind::InvalidValue)
    })
}

#[derive(Subcommand, Debug, Clone)]
pub enum ServerMode {
    Plaintext {
        #[arg(help = "Tcp listen port", default_value = "0.0.0.0:9466", value_parser = parse_address)]
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

use std::net::SocketAddr;

#[derive(clap::Parser, Debug, Clone)]
pub struct Args {
    #[arg(long, default_value = "127.0.0.1:9466", env = "HOST")]
    pub host: SocketAddr,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Command {
    #[command(arg_required_else_help = true)]
    Put {
        key: String,
        #[arg(value_parser=parse_value)]
        value: rmemstore::types::MemstoreValue,
    },
    #[command(arg_required_else_help = true)]
    Get { key: String },
}

fn parse_value(s: &str) -> Result<rmemstore::types::MemstoreValue, serde_json::Error> {
    serde_json::from_str(s)
}

use std::{collections::HashMap, net::SocketAddr};

use rmemstore::conversions::IntoValue;


#[derive(clap::Parser, Debug, Clone)]
pub struct Args {
    #[arg(long, default_value="127.0.0.1:9466", env = "HOST")]
    pub host: SocketAddr,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Command {
    #[command(arg_required_else_help = true)]
    Put {
        key: String,
        #[command(subcommand)]
        value: PutValue,
    },
    #[command(arg_required_else_help = true)]
    Get {
        key: String,
    }
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum PutValue {
    #[command(arg_required_else_help = true)]
    Blob {
        value: String,
    },
    Map {
        #[arg(value_parser=parse_map)]
        value: HashMap<String, Value>,
    }
}

fn parse_map(s: &str) -> Result<HashMap<String, Value>, serde_json::Error> {
    serde_json::from_str(s)
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(rename_all="snake_case")]
pub enum Value {
    Blob(String),
    Map(HashMap<String, Value>),
}

impl IntoValue for Value {
    fn into_value(self) -> messages::rmemstore::value::Kind {
        match self {
            Value::Blob(blob) => blob.into_value(),
            Value::Map(map) => map.into_value(),
        }
    }
}

impl IntoValue for PutValue {
    fn into_value(self) -> messages::rmemstore::value::Kind {
        match self {
            PutValue::Blob { value } => value.into_value(),
            PutValue::Map { value } => value.into_value(),
        }
    }
}

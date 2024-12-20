use args::Args;
use bytes::Buf;
use clap::Parser;

mod args;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::Args::parse();
    env_logger::Builder::from_env(env_logger::Env::default().default_write_style_or("always"))
        .init();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("can make a runtime");

    runtime.block_on(run(args))
}

async fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let mut configuration = rmemstore::ConnectionConfiguration::default();
    configuration.max_message_size(32 * (1 << 20));
    let client = rmemstore::Client::connect(
        args.host
            .to_string()
            .parse()
            .expect("host must be a socket address"),
        configuration,
    )
    .await?;

    match args.command {
        args::Command::Put { key, value } => {
            client.put(key, value).await?;
        }
        args::Command::Get { key } => {
            let result = match client.get(key).await? {
                Some(hit) => hit,
                None => {
                    eprintln!("miss");
                    return Ok(());
                }
            };
            match result {
                rmemstore::types::MemstoreValue::Blob { value } => {
                    match std::io::read_to_string(value.reader()) {
                        Ok(v) => {
                            println!("{v}");
                        }
                        Err(e) => {
                            // cli doesn't support unstringable values
                            eprintln!("unsupported value: {e:?}");
                        }
                    }
                }
                rmemstore::types::MemstoreValue::String { string: value } => {
                    println!("{value}")
                }
                rmemstore::types::MemstoreValue::Map { map } => {
                    serde_json::to_writer_pretty(std::io::stdout(), &map)
                        .expect("must be printable");
                    println!();
                }
            }
        }
    }

    Ok(())
}

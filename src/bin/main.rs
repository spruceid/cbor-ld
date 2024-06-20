use clap::Parser;
use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
    process::ExitCode,
    str::FromStr,
};

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    Encode {
        /// Input file.
        input: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    env_logger::init();

    match run(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log::error!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn read_input(input: Option<PathBuf>) -> Result<String, io::Error> {
    match input {
        Some(path) => fs::read_to_string(path),
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

async fn run(args: Args) -> Result<(), Error> {
    let loader = json_ld::loader::ReqwestLoader::new();

    match args.command {
        Command::Encode { input } => {
            let input = read_input(input)?;
            let json = cbor_ld::JsonValue::from_str(&input)?;
            let cbor = cbor_ld::encode(&json, loader).await?;
            ciborium::into_writer(&cbor, std::io::stdout())?;
        }
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error("invalid JSON: {0}")]
    Json(#[from] json_ld::syntax::parse::Error),

    #[error("encoding failed: {0}")]
    Encode(#[from] cbor_ld::EncodeError),

    #[error(transparent)]
    Write(#[from] ciborium::ser::Error<io::Error>),
}

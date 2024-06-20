use clap::Parser;
use iref::IriBuf;
use json_ld::{syntax::Parse, ChainLoader, FsLoader, Print, ReqwestLoader};
use std::{
    fs,
    io::{self, stdout, Read, Write},
    path::PathBuf,
    process::ExitCode,
    str::FromStr,
};

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,

    /// Add a local mount point for the context loader.
    ///
    /// The value must be of the form `iri=path`.
    /// When fetching a JSON-LD context, if the context URL starts with `iri`,
    /// the context file will be loaded from the file system at `path`.
    #[clap(short, long, global = true)]
    mount: Vec<Mount>,

    /// Local-only context loader.
    ///
    /// Enabling this option will disable remote context fetching.
    #[clap(short, long, global = true)]
    local: bool,
}

#[derive(clap::Subcommand)]
enum Command {
    Encode {
        /// Input file.
        input: Option<PathBuf>,

        /// Enable hexadecimal encoding.
        #[clap(short = 'x', long)]
        hexadecimal: bool,
    },

    Decode {
        /// Input file.
        input: Option<PathBuf>,

        /// Parse the input file has hexadecimal-encoded.
        #[clap(short = 'x', long)]
        hexadecimal: bool,
    },
}

#[derive(Debug, thiserror::Error)]
#[error("invalid mount value")]
struct InvalidMountValue;

#[derive(Debug, Clone)]
struct Mount {
    iri: IriBuf,
    path: PathBuf,
}

impl FromStr for Mount {
    type Err = InvalidMountValue;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (prefix, value) = s.split_once('=').ok_or(InvalidMountValue)?;
        Ok(Self {
            iri: IriBuf::new(prefix.to_owned()).map_err(|_| InvalidMountValue)?,
            path: value.into(),
        })
    }
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

fn read_input(input: Option<PathBuf>) -> Result<Vec<u8>, io::Error> {
    match input {
        Some(path) => fs::read(path),
        None => {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            Ok(buffer)
        }
    }
}

async fn run(args: Args) -> Result<(), Error> {
    let mut fs_loader = FsLoader::new();

    for m in args.mount {
        fs_loader.mount(m.iri, m.path);
    }

    let loader = if args.local {
        Loader::Local(fs_loader)
    } else {
        Loader::Remote(ChainLoader::new(fs_loader, ReqwestLoader::new()))
    };

    match args.command {
        Command::Encode { input, hexadecimal } => {
            let input = read_input(input)?;
            let json = cbor_ld::JsonValue::parse_slice(&input)?.0;
            let bytes = cbor_ld::encode_to_bytes(&json, loader).await?;

            if hexadecimal {
                let hex_bytes = hex::encode(&bytes).into_bytes();
                stdout().write_all(&hex_bytes)?
            } else {
                stdout().write_all(&bytes)?
            }
        }
        Command::Decode { input, hexadecimal } => {
            let bytes = if hexadecimal {
                read_input(input)?
            } else {
                let hex_bytes = read_input(input)?;
                hex::decode(hex_bytes)?
            };

            let json = cbor_ld::decode_from_bytes(&bytes, loader).await?;
            eprintln!("{}", json.pretty_print())
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

    #[error(transparent)]
    Hex(#[from] hex::FromHexError),

    #[error("encoding failed: {0}")]
    Encode(#[from] cbor_ld::EncodeError),

    #[error("decoding failed: {0}")]
    Decode(#[from] cbor_ld::DecodeError),

    #[error(transparent)]
    Write(#[from] ciborium::ser::Error<io::Error>),
}

enum Loader {
    Local(FsLoader<IriBuf>),
    Remote(ChainLoader<FsLoader<IriBuf>, ReqwestLoader>),
}

#[derive(Debug, thiserror::Error)]
enum LoaderError {
    #[error(transparent)]
    Local(json_ld::loader::fs::Error),

    #[error(transparent)]
    Remote(
        json_ld::loader::chain::Error<json_ld::loader::fs::Error, json_ld::loader::reqwest::Error>,
    ),
}

impl json_ld::Loader<IriBuf> for Loader {
    type Error = LoaderError;

    async fn load_with<V>(
        &mut self,
        vocabulary: &mut V,
        url: IriBuf,
    ) -> json_ld::LoadingResult<IriBuf, Self::Error>
    where
        V: rdf_types::vocabulary::IriVocabularyMut<Iri = IriBuf>,
    {
        match self {
            Self::Local(l) => l
                .load_with(vocabulary, url)
                .await
                .map_err(LoaderError::Local),
            Self::Remote(l) => l
                .load_with(vocabulary, url)
                .await
                .map_err(LoaderError::Remote),
        }
    }
}

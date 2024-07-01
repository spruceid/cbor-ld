use cbor_ld::{contexts::REGISTERED_CONTEXTS, DecodeOptions, EncodeOptions, IdMap};
use clap::Parser;
use iref::{Iri, IriBuf, IriRefBuf};
use json_ld::{syntax::Parse, ChainLoader, FsLoader, Print, ReqwestLoader};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
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

    #[clap(flatten)]
    config: Configuration,

    /// Configuration file.
    #[clap(short = 't', long = "config")]
    config_files: Vec<PathBuf>,
}

#[derive(clap::Args, Deserialize)]
struct Configuration {
    /// Add a local mount point for the context loader.
    ///
    /// The value must be of the form `iri=path`.
    /// When fetching a JSON-LD context, if the context URL starts with `iri`,
    /// the context file will be loaded from the file system at `path`.
    #[clap(short, long, global = true)]
    #[serde(default, deserialize_with = "deserialize_mount_map")]
    mount: Vec<Mount>,

    /// Add an application-specific context ID.
    ///
    /// The value must be of the form `iri-reference=id` where id is an
    /// application-specific non-negative integer identifier for the JSON-LD
    /// context identifier by the given IRI reference.
    #[clap(short, long = "context", global = true)]
    #[serde(default, deserialize_with = "deserialize_context_map")]
    contexts: Vec<ContextDefinition>,

    /// Offline context loader.
    ///
    /// Enabling this option will disable remote context fetching.
    #[clap(short, long, global = true)]
    #[serde(default)]
    offline: bool,
}

impl Configuration {
    fn extend(&mut self, other: Self) {
        self.mount.extend(other.mount);
        self.contexts.extend(other.contexts);
        self.offline |= other.offline;
    }
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

fn deserialize_mount_map<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Mount>, D::Error> {
    BTreeMap::<IriBuf, PathBuf>::deserialize(deserializer).map(|map| {
        map.into_iter()
            .map(|(iri, path)| Mount { iri, path })
            .collect()
    })
}

#[derive(Debug, thiserror::Error)]
#[error("invalid context ID definition")]
struct InvalidContextDefinition;

#[derive(Debug, Clone)]
struct ContextDefinition {
    id: u64,
    iri_ref: IriRefBuf,
}

impl FromStr for ContextDefinition {
    type Err = InvalidContextDefinition;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (iri_ref, id) = s.split_once('=').ok_or(InvalidContextDefinition)?;
        Ok(Self {
            iri_ref: IriRefBuf::new(iri_ref.to_owned()).map_err(|_| InvalidContextDefinition)?,
            id: id.parse().map_err(|_| InvalidContextDefinition)?,
        })
    }
}

fn deserialize_context_map<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<ContextDefinition>, D::Error> {
    BTreeMap::<IriRefBuf, u64>::deserialize(deserializer).map(|map| {
        map.into_iter()
            .map(|(iri_ref, id)| ContextDefinition { iri_ref, id })
            .collect()
    })
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

    let mut config = args.config;

    for path in args.config_files {
        let content = fs::read_to_string(path)?;
        let c: Configuration = toml::from_str(&content)?;
        config.extend(c)
    }

    for m in config.mount {
        fs_loader.mount(m.iri, m.path);
    }

    let loader = if config.offline {
        Loader::Offline(fs_loader)
    } else {
        Loader::Online(ChainLoader::new(fs_loader, ReqwestLoader::new()))
    };

    let mut context_map = IdMap::new_derived(Some(&REGISTERED_CONTEXTS));

    for d in config.contexts {
        context_map.insert(d.iri_ref.into_string(), d.id);
    }

    match args.command {
        Command::Encode { input, hexadecimal } => {
            let input = read_input(input)?;
            let json = cbor_ld::JsonValue::parse_slice(&input)?.0;

            let options = EncodeOptions {
                context_map,
                ..Default::default()
            };

            let bytes = cbor_ld::encode_to_bytes_with(&json, loader, options).await?;

            if hexadecimal {
                let hex_bytes = hex::encode(&bytes).into_bytes();
                stdout().write_all(&hex_bytes)?
            } else {
                stdout().write_all(&bytes)?
            }
        }
        Command::Decode { input, hexadecimal } => {
            let bytes = if hexadecimal {
                let hex_bytes = read_input(input)?;
                hex::decode(hex_bytes)?
            } else {
                read_input(input)?
            };

            let options = DecodeOptions {
                context_map,
                ..Default::default()
            };

            let json = cbor_ld::decode_from_bytes_with(&bytes, loader, options).await?;
            eprintln!("{}", json.pretty_print())
        }
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error("unable to read TOML configuration file: {0}")]
    Toml(#[from] toml::de::Error),

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
    Offline(FsLoader),
    Online(ChainLoader<FsLoader, ReqwestLoader>),
}

impl json_ld::Loader for Loader {
    async fn load(&self, url: &Iri) -> json_ld::LoadingResult<IriBuf> {
        match self {
            Self::Offline(l) => l.load(url).await,
            Self::Online(l) => l.load(url).await,
        }
    }
}

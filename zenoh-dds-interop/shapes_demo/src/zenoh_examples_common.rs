//! Common CLI args shared across Zenoh examples.
use serde_json::json;
use zenoh::config::Config;

#[derive(clap::ValueEnum, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Wai {
    Peer,
    Client,
    Router,
}
impl core::fmt::Display for Wai {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        core::fmt::Debug::fmt(&self, f)
    }
}
#[derive(clap::Parser, Clone, PartialEq, Eq, Hash, Debug)]
pub struct CommonArgs {
    #[arg(short, long)]
    /// A configuration file.
    config: Option<String>,
    #[arg(short, long)]
    /// The Zenoh session mode [default: peer].
    mode: Option<Wai>,
    #[arg(short = 'e', long)]
    /// Endpoints to connect to.
    connect: Vec<String>,
    #[arg(short, long)]
    /// Endpoints to listen on.
    listen: Vec<String>,
    #[arg(long)]
    /// Disable the multicast-based scouting mechanism.
    no_multicast_scouting: bool,
    #[arg(long)]
    /// Enable shared-memory feature.
    enable_shm: bool,
}

impl From<CommonArgs> for Config {
    fn from(value: CommonArgs) -> Self {
        (&value).into()
    }
}
impl From<&CommonArgs> for Config {
    fn from(value: &CommonArgs) -> Self {
        let mut config = match &value.config {
            Some(path) => Config::from_file(path).unwrap(),
            None => Config::default(),
        };
        if let Some(mode) = value.mode {
            config
                .insert_json5("mode", &json!(mode.to_string().to_lowercase()).to_string())
                .unwrap();
        }
        if !value.connect.is_empty() {
            config
                .insert_json5("connect/endpoints", &json!(value.connect).to_string())
                .unwrap();
        }
        if !value.listen.is_empty() {
            config
                .insert_json5("listen/endpoints", &json!(value.listen).to_string())
                .unwrap();
        }
        if value.no_multicast_scouting {
            config
                .insert_json5("scouting/multicast/enabled", "false")
                .unwrap();
        }
        if value.enable_shm {
            config
                .insert_json5("transport/shared_memory/enabled", "true")
                .unwrap();
        }
        config
    }
}

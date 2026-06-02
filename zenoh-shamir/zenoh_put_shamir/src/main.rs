use clap::Parser;
use sharks::{Share, Sharks};
use std::convert::TryInto;
use zenoh::Config;

#[derive(Parser, Debug)]
#[command(about = "Zenoh + Shamir put example")]
struct Args {
    #[arg(short, long)]
    mode: Option<String>,
    #[arg(short = 'e', long)]
    connect: Vec<String>,
    #[arg(short, long)]
    listen: Vec<String>,
    #[arg(short, long)]
    config: Option<String>,
    #[arg(long)]
    no_multicast_scouting: bool,
    #[arg(short, long, default_value = "demo/example/zenoh-shamir-put")]
    key: String,
    #[arg(short, long, default_value = "Enigm@")]
    value: String,
    #[arg(short, long, default_value = "2")]
    threshold: u8,
    #[arg(short, long, default_value = "2")]
    redundancy: u8,
}

#[tokio::main]
async fn main() {
    zenoh::init_log_from_env_or("error");
    let args = Args::parse();

    let config = build_config(&args);

    println!("Open zenoh session");
    let session = zenoh::open(config).await.unwrap();

    let sharks = Sharks(args.threshold);
    let dealer = sharks.dealer(args.value.as_bytes());
    let shares: Vec<Share> = dealer
        .take((args.threshold * args.redundancy) as usize)
        .collect();

    let mut normalized_expr = args.key.clone();
    if !normalized_expr.starts_with('/') {
        normalized_expr = format!("/{}", normalized_expr);
    }

    for (index, share) in shares.iter().enumerate() {
        let share_expr = format!("share/{}{}", index, normalized_expr);
        println!("Putting share {} of '{}'. ", index, share_expr);
        let share_as_bytes: Vec<u8> = share.try_into().unwrap();
        session.put(&share_expr, share_as_bytes).await.unwrap();
    }
}

fn build_config(args: &Args) -> Config {
    use serde_json::json;
    let mut config = match &args.config {
        Some(path) => Config::from_file(path).unwrap(),
        None => Config::default(),
    };
    if let Some(mode) = &args.mode {
        config
            .insert_json5("mode", &json!(mode).to_string())
            .unwrap();
    }
    if !args.connect.is_empty() {
        config
            .insert_json5("connect/endpoints", &json!(args.connect).to_string())
            .unwrap();
    }
    if !args.listen.is_empty() {
        config
            .insert_json5("listen/endpoints", &json!(args.listen).to_string())
            .unwrap();
    }
    if args.no_multicast_scouting {
        config
            .insert_json5("scouting/multicast/enabled", "false")
            .unwrap();
    }
    config
}

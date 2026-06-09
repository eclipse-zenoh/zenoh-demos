use clap::Parser;
use sharks::Sharks;
use zenoh::Config;

#[derive(Parser, Debug)]
#[command(about = "Zenoh + Shamir queryable example")]
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
    #[arg(short, long, default_value = "demo/example/zenoh-shamir-queryable")]
    key: String,
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
    let threshold = args.threshold;
    let redundancy = args.redundancy;
    let key_expr = args.key.clone();

    println!("Open zenoh session");
    let session = zenoh::open(config).await.unwrap();

    let queryable = session.declare_queryable(&key_expr).await.unwrap();
    let sharks = Sharks(threshold);

    while let Ok(query) = queryable.recv_async().await {
        println!(
            ">> [zenoh_queryable_shamir] received query: {}",
            query.key_expr()
        );

        let name = query
            .parameters()
            .get("name")
            .unwrap_or("Rust!")
            .to_string();

        let secret = if name.starts_with('/') {
            let mut shares: Vec<sharks::Share> = Vec::with_capacity(threshold as usize);
            let mut index = 0u8;
            while shares.len() < threshold as usize && index < threshold * redundancy {
                let share_expr = format!("share/{}{}", index, name);
                print!("\t>> Fetching share '{}': ", share_expr);
                if let Some(share) = get_share(&session, &share_expr).await {
                    shares.push(share);
                    println!("OK.");
                } else {
                    println!("not found.");
                }
                index += 1;
            }

            if shares.len() < threshold as usize {
                let msg = format!(
                    "Not enough shares ({}/{})",
                    shares.len(),
                    threshold
                );
                println!("\t>> {}. Aborting.", msg);
                msg
            } else {
                let secret = String::from_utf8(sharks.recover(&shares).unwrap()).unwrap();
                println!("\t>> Sending back reconstructed secret.");
                secret
            }
        } else {
            println!("\t>> Expected a key expression starting with '/'.");
            "Error: key must start with '/'".to_string()
        };

        query.reply(query.key_expr(), secret).await.unwrap();
    }
}

async fn get_share(session: &zenoh::Session, path: &str) -> Option<sharks::Share> {
    let replies = session.get(path).await.unwrap();
    while let Ok(reply) = replies.recv_async().await {
        if let Ok(sample) = reply.into_result() {
            let bytes = sample.payload().to_bytes().to_vec();
            return Some(sharks::Share::try_from(bytes.as_ref()).unwrap());
        }
    }
    None
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

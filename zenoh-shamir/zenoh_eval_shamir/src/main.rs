use clap::{App, Arg};
use futures::prelude::*;
use sharks::Sharks;
use std::convert::TryFrom;
use zenoh::*;

#[async_std::main]
async fn main() {
    env_logger::init();

    let (config, path, threshold, redundancy) = parse_args();

    let path = &Path::try_from(path).unwrap();

    let zenoh = Zenoh::new(config.into()).await.unwrap();

    let workspace = zenoh.workspace(None).await.unwrap();

    let mut get_stream = workspace.register_eval(&path.into()).await.unwrap();

    let sharks = Sharks(threshold);

    while let Some(get_request) = get_stream.next().await {
        println!(
            ">> [zenoh_eval_shamir listener] received get with selector: {}",
            get_request.selector
        );

        let name = get_request
            .selector
            .properties
            .get("name")
            .cloned()
            .unwrap_or_else(|| "Rust!".to_string());

        let mut secret = "Error".to_string();

        if name.starts_with('/') {
            let mut shares: Vec<sharks::Share> = Vec::with_capacity(threshold as usize);
            let mut index = 0;
            while shares.len() < threshold as usize && index < threshold * redundancy {
                let share_path = format!("/share/{}{}", index, name);
                print!("\t>> [zenoh_eval_shamir] Fetching share '{}': ", share_path);
                if let Some(share) = get_share(&workspace, &share_path).await {
                    shares.push(share);
                    println!(" OK.");
                }

                index += 1;
            }

            if shares.len() < threshold as usize {
                secret = format!(
                    "Not enough shares were retrieved ({}/{})",
                    shares.len(),
                    threshold
                );
                println!("\t>> [zenoh_eval_shamir] {}. Aborting.", secret);
            } else {
                // Reconstruct the secret
                secret = String::from_utf8(sharks.recover(&shares).unwrap()).unwrap();
                println!("\t>> [zenoh_eval_shamir] Sending back reconstructed secret.");
            }
        } else {
            println!("\t>> [zenoh_eval_shamir] A path starting with a '/' is expected.");
        }

        get_request.reply(path.clone(), secret.into()).await;
    }

    get_stream.close().await.unwrap();
    zenoh.close().await.unwrap();
}

async fn get_share(workspace: &Workspace<'_>, path: &str) -> Option<sharks::Share> {
    let mut share: Option<sharks::Share> = None;

    if let Ok(selector) = Selector::try_from(path) {
        match workspace.get(&selector).await.unwrap().next().await {
            Some(Data {
                path: _,
                value: Value::Raw(0, v),
                timestamp: _,
            }) => {
                let v_bytes = v.get_vec();
                share = Some(sharks::Share::try_from(v_bytes.as_slice()).unwrap());
            }
            Some(_) => println!("Failed to get share '{}'", path),
            None => println!("Failed to get share '{}': not found", path),
        }
    } else {
        println!("Failed to get value from '{}': not a valid Selector", path);
    }

    share
}

fn parse_args() -> (Properties, String, u8, u8) {
    let args = App::new("zenoh + shamir eval example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE] 'The zenoh session mode (peer by default).")
                .possible_values(&["peer", "client"]),
        )
        .arg(Arg::from_usage(
            "-e, --peer=[LOCATOR]...  'Peer locators used to initiate the zenoh session.'",
        ))
        .arg(Arg::from_usage(
            "-l, --listener=[LOCATOR]...   'Locators to listen on.'",
        ))
        .arg(
            Arg::from_usage("-t, --threshold=[INTEGER]...   'The numbers of different shares needed to reconstruct the secret.'")
                .default_value("2")
        )
        .arg(
            Arg::from_usage("-r, --redundancy=[INTEGER]...   'The redundancy for each share (the total number of share is thus equal to threshold × redundancy).'")
                .default_value("2")
        )
        .arg(Arg::from_usage(
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .arg(
            Arg::from_usage("-p, --path=[PATH] 'The path the eval will respond to'")
                .default_value("/demo/example/eval-shamir"),
        )
        .arg(Arg::from_usage(
            "--no-multicast-scouting 'Disable the multicast-based scouting mechanism.'",
        ))
        .get_matches();

    let mut config = if let Some(conf_file) = args.value_of("config") {
        Properties::try_from(std::path::Path::new(conf_file)).unwrap()
    } else {
        Properties::default()
    };
    for key in ["mode", "peer", "listener"].iter() {
        if let Some(value) = args.values_of(key) {
            config.insert(key.to_string(), value.collect::<Vec<&str>>().join(","));
        }
    }
    if args.is_present("no-multicast-scouting") {
        config.insert("multicast_scouting".to_string(), "false".to_string());
    }

    let path = args.value_of("path").unwrap().to_string();
    let threshold: u8 = args.value_of("threshold").unwrap().parse().unwrap();
    let redundancy: u8 = args.value_of("redundancy").unwrap().parse().unwrap();

    (config, path, threshold, redundancy)
}

use clap::{App, Arg};
use sharks::{Share, Sharks};
use std::convert::{TryFrom, TryInto};
use std::str;
use zenoh::*;

#[async_std::main]
async fn main() {
    env_logger::init();

    let (config, path, value, threshold, redundancy) = parse_args();

    println!("New Zenoh…");
    let zenoh = Zenoh::new(config.into()).await.unwrap();

    println!("New workspace…");
    let workspace = zenoh.workspace(None).await.unwrap();

    // 1. Split the secret in as many shares as necessary
    let sharks = Sharks(threshold);
    let dealer = sharks.dealer(value.as_bytes());
    let shares: Vec<Share> = dealer.take((threshold * redundancy) as usize).collect();

    let mut normalized_path = path.to_owned();
    if !normalized_path.starts_with('/') {
        normalized_path = format!("/{}", normalized_path);
    }

    // 2. Send the shares to the storages
    for (index, share) in shares.iter().enumerate() {
        let path_share = format!("/share/{}{}", index, normalized_path);

        println!("Putting share {} of '{}'. ", index, path_share);
        let share_as_bytes: Vec<u8> = share.try_into().unwrap();
        workspace
            .put(&path_share.try_into().unwrap(), Value::from(share_as_bytes))
            .await
            .unwrap();
    }

    zenoh.close().await.unwrap();
}

fn parse_args() -> (Properties, String, String, u8, u8) {
    let args = App::new("zenoh put example")
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
        .arg(Arg::from_usage(
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .arg(
            Arg::from_usage("-p, --path=[PATH]        'The name of the resource to put.'")
                .default_value("/demo/example/zenoh-shamir"),
        )
        .arg(
            Arg::from_usage("-v, --value=[VALUE]      'The value of the resource to put.'")
                .default_value("Enigm@"),
        )
        .arg(
            Arg::from_usage("-t, --threshold=[INTEGER]...   'The numbers of different shares needed to reconstruct the secret.'")
                .default_value("2")
        )
        .arg(
            Arg::from_usage("-r, --redundancy=[INTEGER]...   'The redundancy for each share (the total number of share is thus equal to threshold × redundancy).'")
                .default_value("2")
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
    let value = args.value_of("value").unwrap().to_string();
    let threshold: u8 = args.value_of("threshold").unwrap().parse().unwrap();
    let redundancy: u8 = args.value_of("redundancy").unwrap().parse().unwrap();

    (config, path, value, threshold, redundancy)
}

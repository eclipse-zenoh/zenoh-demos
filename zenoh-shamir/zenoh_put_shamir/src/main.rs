use clap::{App, Arg};
use sharks::{Share, Sharks};
use std::convert::TryInto;
use zenoh::config::Config;
use zenoh::prelude::sync::SyncResolve;

fn main() {
    env_logger::init();

    let (config, key_expr, value, threshold, redundancy) = parse_args();

    println!("Open zenoh session");
    let session = zenoh::open(config).res().unwrap();

    // 1. Split the secret in as many shares as necessary
    let sharks = Sharks(threshold);
    let dealer = sharks.dealer(value.as_bytes());
    let shares: Vec<Share> = dealer.take((threshold * redundancy) as usize).collect();

    let mut normalized_expr = key_expr;
    if !normalized_expr.starts_with('/') {
        normalized_expr = format!("/{}", normalized_expr);
    }

    // 2. Send the shares to the storages
    for (index, share) in shares.iter().enumerate() {
        let share_expr = format!("share/{}{}", index, normalized_expr);

        println!("Putting share {} of '{}'. ", index, share_expr);
        let share_as_bytes: Vec<u8> = share.try_into().unwrap();
        session.put(&share_expr, share_as_bytes).res().unwrap();
    }

    session.close().res().unwrap();
}

fn parse_args() -> (Config, String, String, u8, u8) {
    let args = App::new("zenoh + shamir put example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE]  'The zenoh session mode (peer by default).")
                .possible_values(["peer", "client"]),
        )
        .arg(Arg::from_usage(
            "-e, --connect=[ENDPOINT]...   'Endpoints to connect to.'",
        ))
        .arg(Arg::from_usage(
            "-l, --listen=[ENDPOINT]...   'Endpoints to listen on.'",
        ))
        .arg(Arg::from_usage(
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .arg(Arg::from_usage(
            "--no-multicast-scouting 'Disable the multicast-based scouting mechanism.'",
        ))
        .arg(
            Arg::from_usage("-k, --key=[KEYEXPR]        'The key expression to write.'")
                .default_value("demo/example/zenoh-shamir-put"),
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
        .get_matches();

    let mut config = if let Some(conf_file) = args.value_of("config") {
        Config::from_file(conf_file).unwrap()
    } else {
        Config::default()
    };
    if let Some(Ok(mode)) = args.value_of("mode").map(|mode| mode.parse()) {
        config.set_mode(Some(mode)).unwrap();
    }
    if let Some(values) = args.values_of("connect") {
        config
            .connect
            .endpoints
            .extend(values.map(|v| v.parse().unwrap()))
    }
    if let Some(values) = args.values_of("listen") {
        config
            .listen
            .endpoints
            .extend(values.map(|v| v.parse().unwrap()))
    }
    if args.is_present("no-multicast-scouting") {
        config.scouting.multicast.set_enabled(Some(false)).unwrap();
    }

    let key_expr = args.value_of("key").unwrap().to_string();
    let value = args.value_of("value").unwrap().to_string();
    let threshold: u8 = args.value_of("threshold").unwrap().parse().unwrap();
    let redundancy: u8 = args.value_of("redundancy").unwrap().parse().unwrap();

    (config, key_expr, value, threshold, redundancy)
}

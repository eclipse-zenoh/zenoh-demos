use clap::{App, Arg};
use sharks::Sharks;
use std::convert::TryFrom;
use zenoh::config::Config;
use zenoh::prelude::*;

fn main() {
    env_logger::init();

    let (config, key_expr, threshold, redundancy) = parse_args();

    println!("Open zenoh session");
    let session = zenoh::open(config).wait().unwrap();

    let mut queryable = session
        .queryable(&key_expr)
        .kind(zenoh::queryable::EVAL)
        .wait()
        .unwrap();

    let sharks = Sharks(threshold);

    while let Ok(query) = queryable.receiver().recv() {
        println!(
            ">> [zenoh_eval_shamir listener] received query with selector: {}",
            query.selector()
        );

        let name = query
            .selector()
            .parse_value_selector()
            .unwrap()
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
                if let Some(share) = get_share(&session, &share_path) {
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

        query.reply(Sample::new(key_expr.clone(), secret));
    }

    queryable.close().wait().unwrap();
    session.close().wait().unwrap();
}

fn get_share(session: &zenoh::Session, path: &str) -> Option<sharks::Share> {
    let mut share: Option<sharks::Share> = None;

    if let Ok(selector) = Selector::try_from(path) {
        match session.get(&selector).wait().unwrap().recv() {
            Ok(reply) => {
                let v_bytes = reply.sample.value.payload.contiguous();
                share = Some(sharks::Share::try_from(v_bytes.as_ref()).unwrap());
            }
            Err(_) => println!("Failed to get share '{}': not found", path),
        }
    } else {
        println!("Failed to get value from '{}': not a valid Selector", path);
    }

    share
}

fn parse_args() -> (Config, String, u8, u8) {
    let args = App::new("zenoh + shamir eval example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE]  'The zenoh session mode (peer by default).")
                .possible_values(&["peer", "client"]),
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
            Arg::from_usage("-k, --key=[KEYEXPR]        'The key expression matching queries to evaluate.'")
                .default_value("/demo/example/zenoh-shamir-eval"),
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
    let threshold: u8 = args.value_of("threshold").unwrap().parse().unwrap();
    let redundancy: u8 = args.value_of("redundancy").unwrap().parse().unwrap();

    (config, key_expr, threshold, redundancy)
}

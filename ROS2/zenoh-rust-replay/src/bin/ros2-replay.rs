//
// Copyright (c) 2021 ADLINK Technology Inc.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ADLINK zenoh team, <zenoh@adlink-labs.tech>
//
use async_std::task;
use cdr::Infinite;
use clap::{App, Arg};
use futures::prelude::*;
use serde_derive::Deserialize;
use zenoh::config::Config;
use zenoh::net::protocol::io::SplitBuffer;
use zenoh::query::*;
use zenoh::queryable;

#[derive(Deserialize, Debug, PartialEq)]
struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Deserialize, Debug, PartialEq)]
struct Twist {
    linear: Vector3,
    angular: Vector3,
}

#[async_std::main]
async fn main() {
    // Initiate logging
    env_logger::init();

    let (config, cmd_vel, iscope, oscope, filter) = parse_args();

    println!("Opening session...");
    let session = zenoh::open(config).await.unwrap();

    let mut input = iscope;
    input.push_str(cmd_vel.as_str());
    if !input.is_empty() {
        input.push_str("?");
        input.push_str(filter.as_str());
    }

    let mut output = oscope;
    output.push_str(cmd_vel.as_str());

    println!("Sending Query '{}'...", input);
    let tgt = QueryTarget {
        kind: queryable::STORAGE,
        target: Target::All,
    };
    let mut replies = session
        .get(&input)
        .target(tgt)
        .await
        .unwrap()
        .collect::<Vec<Reply>>()
        .await;
    replies.sort_by(|a, b| a.data.timestamp.partial_cmp(&b.data.timestamp).unwrap());
    let mut ts = None;
    for reply in replies {
        let now = match (ts, reply.data.timestamp) {
            (Some(t1), Some(t2)) => {
                task::sleep(t2.get_diff_duration(&t1)).await;
                t2
            }
            (None, Some(t2)) => t2,
            _ => panic!(),
        };
        ts = reply.data.timestamp;

        let cmd =
            cdr::deserialize_from::<_, Twist, _>(&*reply.data.value.payload.contiguous(), Infinite)
                .unwrap();
        println!(
            "[{}] Replay '{}': '{:?}'",
            now.get_time(),
            reply.data.key_expr.as_str(),
            cmd
        );
        session
            .put(output.as_str(), reply.data.value.payload)
            .await
            .unwrap();
    }
}

fn parse_args() -> (Config, String, String, String, String) {
    let args = App::new("zenoh-net sub example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE] 'The zenoh session mode (peer by default).")
                .possible_values(&["peer", "client"]),
        )
        .arg(Arg::from_usage(
            "-e, --connect=[LOCATOR]... 'Locators to connect to.'",
        ))
        .arg(Arg::from_usage(
            "-c, --config=[FILE] 'A configuration file.'",
        ))
        .arg(
            Arg::from_usage("--cmd_vel=[topic] 'The 'cmd_vel' ROS2 topic.'")
                .default_value("/rt/turtle1/cmd_vel"),
        )
        .arg(
            Arg::from_usage("-f, --filter=[String] 'The 'filter' for querying. E.g. \"starttime=now()-1m;stoptime=now()\"'")
                .default_value("(starttime=0)"),
        )
        .arg(
            Arg::from_usage("-i, --input-scope=[String] 'A string added as prefix to all routed DDS topics when mapped to a zenoh resource.'")
                .default_value(""),
        )
        .arg(
            Arg::from_usage("-o, --output-scope=[String] 'A string added as prefix to all routed DDS topics when mapped to a zenoh resource.'")
                .required(true),
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

    let cmd_vel = args.value_of("cmd_vel").unwrap().to_string();
    let iscope = args.value_of("input-scope").unwrap().to_string();
    let oscope = args.value_of("output-scope").unwrap().to_string();
    let filter = args.value_of("filter").unwrap().to_string();

    (config, cmd_vel, iscope, oscope, filter)
}

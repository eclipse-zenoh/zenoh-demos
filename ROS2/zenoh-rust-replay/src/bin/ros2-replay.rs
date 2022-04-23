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
use cdr::CdrLe;
use cdr::Infinite;
use clap::{App, Arg};
use futures::prelude::*;
use serde_derive::{Deserialize, Serialize};
use zenoh::buf::reader::HasReader;
use zenoh::buf::ZBuf;
use zenoh::config::Config;
use zenoh::query::*;
use zenoh::queryable;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Twist {
    linear: Vector3,
    angular: Vector3,
}

#[async_std::main]
async fn main() {
    // Initiate logging
    env_logger::init();

    let (config, query_selector, pub_expr, time_scale, is_twist, angular_scale, linear_scale) =
        parse_args();

    println!("Opening session...");
    let session = zenoh::open(config).await.unwrap();

    // Get stored publications
    println!("Sending Query '{}'...", query_selector);
    let tgt = QueryTarget {
        kind: queryable::STORAGE,
        target: Target::All,
    };
    let mut replies = session
        .get(&query_selector)
        .target(tgt)
        .await
        .unwrap()
        .collect::<Vec<Reply>>()
        .await;
    // Sort publications by timestamps
    replies.sort_by(|a, b| a.sample.timestamp.partial_cmp(&b.sample.timestamp).unwrap());

    if replies.is_empty() {
        println!("No publication found - nothing to replay.");
        return;
    }

    let first_ts = replies.first().unwrap().sample.timestamp.unwrap();
    let last_ts = replies.last().unwrap().sample.timestamp.unwrap();
    println!(
        "Replay {} publications that were made between {} and {} ",
        replies.len(),
        first_ts.get_time(),
        last_ts.get_time(),
    );
    println!(
        "Initial duration: {} seconds => with time-scale={}, new duration: {} seconds",
        last_ts.get_diff_duration(&first_ts).as_secs_f32(),
        time_scale,
        last_ts
            .get_diff_duration(&first_ts)
            .mul_f64(time_scale)
            .as_secs_f32(),
    );

    let mut ts = None;
    for reply in replies {
        // compute time difference and sleep (*time_scale)
        let now = match (ts, reply.sample.timestamp) {
            (Some(t1), Some(t2)) => {
                task::sleep(t2.get_diff_duration(&t1).mul_f64(time_scale)).await;
                t2
            }
            (None, Some(t2)) => t2,
            _ => panic!(),
        };
        ts = reply.sample.timestamp;

        println!(
            "[{}] Replay publication from '{}' to '{}'",
            now.get_time(),
            reply.sample.key_expr,
            pub_expr
        );

        if is_twist {
            // payload is a Twist, apply scales and replay
            let new_payload =
                transform_twist(&reply.sample.value.payload, linear_scale, angular_scale);
            session.put(pub_expr.as_str(), new_payload).await.unwrap();
        } else {
            // replay payload unchanged
            session
                .put(pub_expr.as_str(), reply.sample.value.payload)
                .await
                .unwrap();
        }
    }
}

fn transform_twist(payload: &ZBuf, linear_scale: f64, angular_scale: f64) -> Vec<u8> {
    let mut twist = cdr::deserialize_from::<_, Twist, _>(payload.reader(), Infinite).unwrap();
    twist.linear.x *= linear_scale;
    twist.linear.y *= linear_scale;
    twist.linear.z *= linear_scale;
    twist.angular.x *= angular_scale;
    twist.angular.y *= angular_scale;
    twist.angular.z *= angular_scale;
    println!("   '{:?}'", twist);
    cdr::serialize::<_, _, CdrLe>(&twist, Infinite).unwrap()
}

fn parse_args() -> (Config, String, String, f64, bool, f64, f64) {
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
            Arg::from_usage("-f, --filter=[String] 'The 'filter' for querying. E.g. \"starttime=now()-1m;stoptime=now()\"'")
                .default_value("(starttime=0)"),
        )
        .arg(
            Arg::from_usage("-i, --input-path=[String] 'A complete overwrite of the input zenoh resrouce (option -i will be ignored).'")
                .default_value("/rt/turtle1/cmd_vel"),
        )
        .arg(
            Arg::from_usage("-o, --output-path=[String] 'A complete overwrite of the output zenoh resrouce (option -o will be ignored).'")
                .default_value("/replay/rt/turtle1/cmd_vel"),
        )
        .arg(Arg::from_usage("-t, --time-scale=[FLOAT] 'The time scale (i.e. multiplier of time interval between each re-publication).").default_value("1.0"))
        .arg(Arg::from_usage("-w, --twist 'The data is a ROS2 Twist message. --angular-scale and --linear-scale will appli"))
        .arg(
            Arg::from_usage("-a, --angular-scale=[FLOAT] 'The angular scale (apply only if  --twist is set).'")
                .default_value("1.0"),
        )
        .arg(Arg::from_usage("-x, --linear-scale=[FLOAT] 'The linear scale (apply only if  --twist is set).").default_value("1.0"))
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

    let ipath = args.value_of("input-path").unwrap().to_string();
    let opath = args.value_of("output-path").unwrap().to_string();
    let filter = args.value_of("filter").unwrap().to_string();
    let time_scale: f64 = args.value_of("time-scale").unwrap().parse().unwrap();
    let is_twist = args.is_present("twist");
    let angular_scale: f64 = args.value_of("angular-scale").unwrap().parse().unwrap();
    let linear_scale: f64 = args.value_of("linear-scale").unwrap().parse().unwrap();

    let mut query_selector = ipath;
    if !filter.is_empty() {
        query_selector.push('?');
        query_selector.push_str(filter.as_str());
    }

    let pub_expr = opath;

    (
        config,
        query_selector,
        pub_expr,
        time_scale,
        is_twist,
        angular_scale,
        linear_scale,
    )
}

//
// Copyright (c) 2021 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   The Zenoh Team, <zenoh@zettascale.tech>
//
use async_std::task;
use cdr::CdrLe;
use cdr::Infinite;
use clap::Parser;
use futures::prelude::*;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use zenoh::{bytes::ZBytes, config::WhatAmI, query::ConsolidationMode, Config};

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
    zenoh::init_log_from_env_or("error");

    let args = Args::parse();
    let config: Config = (&args).into();

    println!("Opening session...");
    let session = zenoh::open(config).await.unwrap();
    let publisher = session.declare_publisher(args.output_topic).await.unwrap();

    let mut query_selector = args.input_topic;
    if !args.filter.is_empty() {
        query_selector.push('?');
        query_selector.push_str(&args.filter);
    }

    // Get stored publications
    println!("Sending Query '{}'...", query_selector);
    let mut replies = session
        .get(&query_selector)
        .consolidation(ConsolidationMode::None)
        .await
        .unwrap()
        .stream()
        .filter_map(|r| async move { r.into_result().ok() })
        .collect::<Vec<_>>()
        .await;

    // Sort publications by timestamps
    replies.sort_by(|a, b| a.timestamp().partial_cmp(&b.timestamp()).unwrap());

    if replies.is_empty() {
        println!("No publications found - nothing to replay.");
        return;
    }

    let first_ts = replies.first().unwrap().timestamp().unwrap();
    let last_ts = replies.last().unwrap().timestamp().unwrap();
    println!(
        "Replay {} publications that were made between {} and {} ",
        replies.len(),
        first_ts.get_time(),
        last_ts.get_time(),
    );
    println!(
        "Initial duration: {} seconds => with time-scale={}, new duration: {} seconds",
        last_ts.get_diff_duration(first_ts).as_secs_f32(),
        args.time_scale,
        last_ts
            .get_diff_duration(first_ts)
            .mul_f64(args.time_scale)
            .as_secs_f32(),
    );

    let mut ts = None;
    for s in replies {
        // compute time difference and sleep (*time_scale)
        let now = match (ts, s.timestamp()) {
            (Some(t1), Some(t2)) => {
                task::sleep(t2.get_diff_duration(&t1).mul_f64(args.time_scale)).await;
                t2
            }
            (None, Some(t2)) => t2,
            _ => panic!(),
        };
        ts = s.timestamp().copied();

        println!(
            "[{}] Replay publication from '{}' to '{}'",
            now.get_time(),
            s.key_expr(),
            publisher.key_expr()
        );
        println!("   {:?}", s.payload());

        if args.twist {
            // payload is a Twist, apply scales and replay
            let new_payload = transform_twist(s.payload(), args.linear_scale, args.angular_scale);
            println!(" ! {:?} ", new_payload);
            publisher.put(new_payload).await.unwrap();
        } else {
            // replay payload unchanged
            publisher.put(s.payload().clone()).await.unwrap();
        }
    }
}

fn transform_twist(payload: &ZBytes, linear_scale: f64, angular_scale: f64) -> Vec<u8> {
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

#[derive(clap::Parser, Clone, Debug)]
pub struct Args {
    #[arg(short, long)]
    /// A configuration file.
    config: Option<String>,
    #[arg(long)]
    /// Allows arbitrary configuration changes as column-separated KEY:VALUE pairs, where:
    ///   - KEY must be a valid config path.
    ///   - VALUE must be a valid JSON5 string that can be deserialized to the expected type for the KEY field.
    ///
    /// Example: `--cfg='transport/unicast/max_links:2'`
    #[arg(long)]
    cfg: Vec<String>,
    #[arg(short, long)]
    /// The Zenoh session mode [default: peer].
    mode: Option<WhatAmI>,
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

    #[arg(short, long, default_value = "_time=[now(-10m)..]")]
    // The 'filter' for querying. E.g. "_time=[now(-1h)..]"
    filter: String,
    #[arg(short, long, default_value = "rt/turtle1/cmd_vel")]
    // The recorded topic to query from storage.
    input_topic: String,
    #[arg(short, long, default_value = "replay/rt/turtle1/cmd_vel")]
    // The topic to replay on.
    output_topic: String,
    #[arg(short, long, default_value = "1.0")]
    // The time scale (i.e. multiplier of time interval between each re-publication).
    time_scale: f64,
    #[arg(short = 'w')]
    // The time scale (i.e. multiplier of time interval between each re-publication).
    twist: bool,
    #[arg(short = 'a', long, default_value = "1.0")]
    // The time scale (i.e. multiplier of time interval between each re-publication).
    angular_scale: f64,
    #[arg(short = 'x', long, default_value = "1.0")]
    // The time scale (i.e. multiplier of time interval between each re-publication).
    linear_scale: f64,
}

impl From<&Args> for Config {
    fn from(args: &Args) -> Self {
        let mut config = match &args.config {
            Some(path) => Config::from_file(path).unwrap(),
            None => Config::default(),
        };
        if let Some(mode) = args.mode {
            config
                .insert_json5("mode", &json!(mode.to_str()).to_string())
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
                .insert_json5("scouting/multicast/enabled", &json!(false).to_string())
                .unwrap();
        }
        if args.enable_shm {
            #[cfg(feature = "shared-memory")]
            config
                .insert_json5("transport/shared_memory/enabled", &json!(true).to_string())
                .unwrap();
            #[cfg(not(feature = "shared-memory"))]
            {
                eprintln!("`--enable-shm` argument: SHM cannot be enabled, because Zenoh is compiled without shared-memory feature!");
                std::process::exit(-1);
            }
        }
        for json in &args.cfg {
            if let Some((key, value)) = json.split_once(':') {
                if let Err(err) = config.insert_json5(key, value) {
                    eprintln!("`--cfg` argument: could not parse `{json}`: {err}");
                    std::process::exit(-1);
                }
            } else {
                eprintln!("`--cfg` argument: expected KEY:VALUE pair, got {json}");
                std::process::exit(-1);
            }
        }
        config
    }
}

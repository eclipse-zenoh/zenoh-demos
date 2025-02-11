//
// Copyright (c) 2017, 2020 ZettaScale Technology
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
use clap::{App, Arg};
use futures::{select, FutureExt};
use opencv::{highgui, prelude::*};
use serde_json::json;
use zenoh::{config::Config, Wait};

#[async_std::main]
async fn main() {
    // initiate logging
    env_logger::init();
    let (config, key_expr) = parse_args();

    println!("Opening session...");
    let session = zenoh::open(config).wait().unwrap();
    let sub = session.declare_subscriber(&key_expr).wait().unwrap();

    let conf_sub = session
        .declare_subscriber(format!("{}/zdisplay/conf/**", key_expr))
        .wait()
        .unwrap();

    loop {
        select!(
            sample = sub.recv_async().fuse() => {
                let sample = sample.unwrap();
                let decoded = opencv::imgcodecs::imdecode(
                    &opencv::types::VectorOfu8::from_slice(&sample.payload().to_bytes()),
                    opencv::imgcodecs::IMREAD_COLOR,
                ).unwrap();

                if decoded.size().unwrap().width > 0 {
                    highgui::imshow(sample.key_expr().as_str(), &decoded).unwrap();
                }

                if highgui::wait_key(10).unwrap() == 113 {
                    // 'q'
                    break;
                }
            },

            sample = conf_sub.recv_async().fuse() => {
                let sample = sample.unwrap();
                let conf_key = sample.key_expr().as_str().split("/conf/").last().unwrap();
                let conf_val = String::from_utf8_lossy(&sample.payload().to_bytes()).to_string();
                let _ = session.config().insert_json5(conf_key, &conf_val);
            },
        );
    }
    conf_sub.undeclare().wait().unwrap();
    sub.undeclare().wait().unwrap();
    session.close().wait().unwrap();
}

fn parse_args() -> (Config, String) {
    let args = App::new("zenoh video display example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE] 'The zenoh session mode.")
                .possible_values(["peer", "client"])
                .default_value("peer"),
        )
        .arg(
            Arg::from_usage("-k, --key=[KEY_EXPR] 'The key expression to subscribe to.")
                .default_value("demo/zcam"),
        )
        .arg(Arg::from_usage(
            "-e, --connect=[LOCATOR]...  'Peer locators used to initiate the zenoh session.'",
        ))
        .arg(Arg::from_usage(
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .get_matches();

    let key_expr = args.value_of("key").unwrap().to_string();

    let mut config = if let Some(conf_file) = args.value_of("config") {
        Config::from_file(conf_file).unwrap()
    } else {
        Config::default()
    };
    if let Some(mode) = args.value_of("mode") {
        config
            .insert_json5("mode", &json!(mode).to_string())
            .unwrap();
    }
    if let Some(peers) = args.values_of("connect") {
        config
            .insert_json5("connect/endpoints", &json!(peers.collect::<Vec<&str>>()).to_string())
            .unwrap();
    }
    (config, key_expr)
}

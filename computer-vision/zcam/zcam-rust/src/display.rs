//
// Copyright (c) 2017, 2020 ADLINK Technology Inc.
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
use clap::{App, Arg};
use futures::prelude::*;
use futures::select;
use opencv::{highgui, prelude::*};
use zenoh::config::Config;
use zenoh::prelude::sync::SyncResolve;
use zenoh::prelude::*;

#[async_std::main]
async fn main() {
    // initiate logging
    env_logger::init();
    let (config, key_expr) = parse_args();

    println!("Openning session...");
    let session = zenoh::open(config).res().unwrap();
    let sub = session.declare_subscriber(&key_expr).res().unwrap();

    let conf_sub = session
        .declare_subscriber(format!("{}/zdisplay/conf/**", key_expr))
        .res()
        .unwrap();

    loop {
        select!(
            sample = sub.recv_async().fuse() => {
                let sample = sample.unwrap();
                let decoded = opencv::imgcodecs::imdecode(
                    &opencv::types::VectorOfu8::from_slice(sample.value.payload.contiguous().as_ref()),
                    opencv::imgcodecs::IMREAD_COLOR,
                ).unwrap();

                if decoded.size().unwrap().width > 0 {
                    highgui::imshow(sample.key_expr.as_str(), &decoded).unwrap();
                }

                if highgui::wait_key(10).unwrap() == 113 {
                    // 'q'
                    break;
                }
            },

            sample = conf_sub.recv_async().fuse() => {
                let sample = sample.unwrap();
                let conf_key = sample.key_expr.as_str().split("/conf/").last().unwrap();
                let conf_val = String::from_utf8_lossy(&sample.value.payload.contiguous()).to_string();
                let _ = session.config().insert_json5(conf_key, &conf_val);
            },
        );
    }
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
            "-e, --peer=[LOCATOR]...  'Peer locators used to initiate the zenoh session.'",
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
    if let Some(Ok(mode)) = args.value_of("mode").map(|mode| mode.parse()) {
        config.set_mode(Some(mode)).unwrap();
    }
    if let Some(peers) = args.values_of("peer") {
        config
            .connect
            .endpoints
            .extend(peers.map(|p| p.parse().unwrap()))
    }
    (config, key_expr)
}

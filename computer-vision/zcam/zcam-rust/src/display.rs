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
use opencv::{highgui, prelude::*, Result};
use serde_json::json;
use zenoh::{config::Config, Wait};

fn main() -> Result<()> {
    // initiate logging
    env_logger::init();
    let (config, key_expr) = parse_args();

    println!("Opening session...");
    let session = zenoh::open(config).wait().unwrap();
    let sub = session.declare_subscriber(&key_expr).wait().unwrap();

    while let Ok(sample) = sub.recv() {
        let decoded = opencv::imgcodecs::imdecode(
            &opencv::types::VectorOfu8::from_slice(sample.payload().to_bytes().as_ref()),
            opencv::imgcodecs::IMREAD_COLOR,
        )?;

        if decoded.size().unwrap().width > 0 {
            highgui::imshow(sample.key_expr().as_str(), &decoded)?;
        }

        if highgui::wait_key(10)? == 113 {
            // 'q'
            break;
        }
    }
    sub.undeclare().wait().unwrap();
    session.close().wait().unwrap();
    Ok(())
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
    if let Some(mode) = args.value_of("mode") {
        config
            .insert_json5("mode", &json!(mode).to_string())
            .unwrap();
    }
    if let Some(peers) = args.values_of("peer") {
        config
            .insert_json5("connect/endpoints", &json!(peers.collect::<Vec<&str>>()).to_string())
            .unwrap();
    }
    (config, key_expr)
}

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
use opencv::{highgui, prelude::*};
use zenoh::config::Config;
use zenoh::net::protocol::io::SplitBuffer;
use zenoh::prelude::*;

fn main() {
    // initiate logging
    env_logger::init();
    let (config, key_expr) = parse_args();

    println!("Openning session...");
    let session = zenoh::open(config).wait().unwrap();
    let mut sub = session.subscribe(&key_expr).wait().unwrap();

    let window = &format!("[{}] Press 'q' to quit.", &key_expr);
    highgui::named_window(window, 1).unwrap();

    while let Ok(sample) = sub.receiver().recv() {
        let decoded = opencv::imgcodecs::imdecode(
            &opencv::types::VectorOfu8::from_iter(sample.value.payload.contiguous().into_owned()),
            opencv::imgcodecs::IMREAD_COLOR,
        )
        .unwrap();

        if decoded.size().unwrap().width > 0 {
            // let mut enlarged = Mat::default().unwrap();
            // opencv::imgproc::resize(&decoded, &mut enlarged, opencv::core::Size::new(800, 600), 0.0, 0.0 , opencv::imgproc::INTER_LINEAR).unwrap();
            highgui::imshow(window, &decoded).unwrap();
        }

        if highgui::wait_key(10).unwrap() == 113 {
            // 'q'
            break;
        }
    }
    sub.close().wait().unwrap();
    session.close().wait().unwrap();
}

fn parse_args() -> (Config, String) {
    let args = App::new("zenoh video display example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE] 'The zenoh session mode.")
                .possible_values(&["peer", "client"])
                .default_value("peer"),
        )
        .arg(
            Arg::from_usage("-k, --key=[KEY_EXPR] 'The key expression to subscribe to.")
                .default_value("/demo/zcam"),
        )
        .arg(Arg::from_usage(
            "-e, --peer=[LOCATOR]...  'Peer locators used to initiate the zenoh session.'",
        ))
        .get_matches();

    let key_expr = args.value_of("key").unwrap().to_string();

    let mut config = Config::default();
    if let Some(Ok(mode)) = args.value_of("mode").map(|mode| mode.parse()) {
        config.set_mode(Some(mode)).unwrap();
    }
    if let Some(peers) = args.values_of("peer") {
        config.peers.extend(peers.map(|p| p.parse().unwrap()))
    }
    (config, key_expr)
}

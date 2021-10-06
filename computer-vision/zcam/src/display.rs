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
use clap::{App, Arg, Values};
use futures::prelude::*;
use futures::select;
use opencv::{highgui, prelude::*};
use zenoh::config;
use zenoh::prelude::*;

#[async_std::main]
async fn main() {
    // initiate logging
    env_logger::init();
    let (config, path) = parse_args();

    println!("Openning session...");
    let session = zenoh::open(config).wait().unwrap();
    let mut sub = session.subscribe(&path).wait().unwrap();
    let mut conf_sub = session
        .subscribe(format!("{}/display/conf/*", path))
        .wait()
        .unwrap();

    let window = &format!("[{}] Press 'q' to quit.", &path);
    highgui::named_window(window, 1).unwrap();

    loop {
        select!(
            sample = sub.receiver().next().fuse() => {
                let sample = sample.unwrap();
                let decoded = opencv::imgcodecs::imdecode(
                    &opencv::types::VectorOfu8::from_iter(sample.value.payload.to_vec()),
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
            },

            sample = conf_sub.receiver().next().fuse() => {
                let sample = sample.unwrap();
                let conf_key = sample.res_name.split('/').last().unwrap();
                let conf_val = String::from_utf8_lossy(&sample.value.payload.contiguous()).to_string();
                let _ = session.config().wait().insert_json(&conf_key, &conf_val);
            },
        );
    }
    sub.unregister().wait().unwrap();
    conf_sub.unregister().wait().unwrap();
    session.close().wait().unwrap();
}

fn parse_args() -> (config::ConfigProperties, String) {
    let args = App::new("zenoh-net video display example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE] 'The zenoh session mode.")
                .possible_values(&["peer", "client"])
                .default_value("peer"),
        )
        .arg(
            Arg::from_usage(
                "-p, --path=[path] 'The zenoh path on which the video will be published.",
            )
            .default_value("/demo/zcam"),
        )
        .arg(Arg::from_usage(
            "-e, --peer=[LOCATOR]...  'Peer locators used to initiate the zenoh session.'",
        ))
        .get_matches();

    let path = args.value_of("path").unwrap();

    let mut config = config::empty();
    config.insert(
        config::ZN_MODE_KEY,
        String::from(args.value_of("mode").unwrap()),
    );
    for peer in args
        .values_of("peer")
        .or_else(|| Some(Values::default()))
        .unwrap()
    {
        config.insert(config::ZN_PEER_KEY, String::from(peer));
    }
    (config, path.to_string())
}

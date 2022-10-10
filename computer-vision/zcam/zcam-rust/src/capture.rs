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
use opencv::{core, prelude::*, videoio};
use zenoh::config::Config;
use zenoh::prelude::sync::SyncResolve;
use zenoh::prelude::*;

#[async_std::main]
async fn main() {
    // initiate logging
    env_logger::init();

    let (config, key_expr, resolution, delay) = parse_args();

    println!("Openning session...");
    let session = zenoh::open(config).res().unwrap();

    let publ = session.declare_publisher(&key_expr).res().unwrap();

    let conf_sub = session
        .declare_subscriber(format!("{}/zcapture/conf/**", key_expr))
        .res()
        .unwrap();

    opencv::opencv_branch_32! {
        let mut cam = videoio::VideoCapture::new_default(0).unwrap(); // 0 is the default camera
    }
    opencv::not_opencv_branch_32! {
        let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap(); // 0 is the default camera
    }

    let opened = videoio::VideoCapture::is_opened(&cam).unwrap();
    if !opened {
        panic!("Unable to open default camera!");
    }
    let mut encode_options = opencv::types::VectorOfi32::new();
    encode_options.push(opencv::imgcodecs::IMWRITE_JPEG_QUALITY);
    encode_options.push(90);

    loop {
        select!(
            _ = async_std::task::sleep(std::time::Duration::from_millis(delay)).fuse() => {
                let mut frame = core::Mat::default();
                cam.read(&mut frame).unwrap();

                if !frame.empty() {
                    let mut reduced = Mat::default();
                    opencv::imgproc::resize(&frame, &mut reduced, opencv::core::Size::new(resolution[0], resolution[1]), 0.0, 0.0 , opencv::imgproc::INTER_LINEAR).unwrap();

                    let mut buf = opencv::types::VectorOfu8::new();
                    opencv::imgcodecs::imencode(".jpeg", &reduced, &mut buf, &encode_options).unwrap();

                    publ.put(buf.to_vec()).res().unwrap();
                } else {
                    println!("Reading empty buffer from camera... Waiting some more....");
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

fn parse_args() -> (Config, String, Vec<i32>, u64) {
    let args = App::new("zenoh videocapture example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE] 'The zenoh session mode.")
                .possible_values(["peer", "client"])
                .default_value("peer"),
        )
        .arg(Arg::from_usage(
            "-e, --connect=[LOCATOR]...  'Endpoints to connect to.'",
        ))
        .arg(
            Arg::from_usage(
                "-k, --key=[KEY_EXPR] 'The key expression on which the video will be published.",
            )
            .default_value("demo/zcam"),
        )
        .arg(
            Arg::from_usage(
                "-r, --resolution=[RESOLUTION] 'The resolution of the published video.",
            )
            .default_value("640x360"),
        )
        .arg(
            Arg::from_usage("-d, --delay=[DELAY] 'The delay between each frame in milliseconds.")
                .default_value("40"),
        )
        .arg(Arg::from_usage(
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .get_matches();

    let mut config = Config::default();
    if let Some(Ok(mode)) = args.value_of("mode").map(|mode| mode.parse()) {
        config.set_mode(Some(mode)).unwrap();
    }
    if let Some(connect) = args.values_of("connect") {
        config
            .connect
            .endpoints
            .extend(connect.map(|p| p.parse().unwrap()))
    }

    let key_expr = args.value_of("key").unwrap().to_string();
    let resolution = args
        .value_of("resolution")
        .unwrap()
        .split('x')
        .map(|s| s.parse::<i32>().unwrap())
        .collect::<Vec<i32>>();
    let delay = args.value_of("delay").unwrap().parse::<u64>().unwrap();

    (config, key_expr, resolution, delay)
}

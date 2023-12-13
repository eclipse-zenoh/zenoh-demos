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
use opencv::{prelude::*, videoio, Result};
use zenoh::config::Config;
use zenoh::prelude::sync::SyncResolve;

fn main() -> Result<()> {
    // initiate logging
    env_logger::init();

    let (config, key_expr, resolution, delay) = parse_args();

    println!("Openning session...");
    let session = zenoh::open(config).res().unwrap();

    let zpub = session.declare_publisher(&key_expr).res().unwrap();

    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;

    let opened = videoio::VideoCapture::is_opened(&cam)?;
    if !opened {
        panic!("Unable to open default camera!");
    }
    let mut encode_options = opencv::types::VectorOfi32::new();
    encode_options.push(opencv::imgcodecs::IMWRITE_JPEG_QUALITY);
    encode_options.push(90);

    loop {
        let mut frame = Mat::default();

        cam.read(&mut frame)?;

        if !frame.empty() {
            let mut reduced = Mat::default();
            opencv::imgproc::resize(
                &frame,
                &mut reduced,
                opencv::core::Size::new(resolution[0], resolution[1]),
                0.0,
                0.0,
                opencv::imgproc::INTER_LINEAR,
            )?;

            let mut buf = opencv::types::VectorOfu8::new();
            opencv::imgcodecs::imencode(".jpeg", &reduced, &mut buf, &encode_options)?;

            zpub.put(buf.to_vec()).res().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(delay));
        } else {
            println!("Reading empty buffer from camera... Waiting some more....");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
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

    let mut config = if let Some(conf_file) = args.value_of("config") {
        Config::from_file(conf_file).unwrap()
    } else {
        Config::default()
    };
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

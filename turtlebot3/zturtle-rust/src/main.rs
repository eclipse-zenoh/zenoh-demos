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
use opencv::{core, prelude::*, videoio};
use serde_derive::Deserialize;
use serde_json::Value;
use std::thread::sleep;
use std::time::Duration;
use zenoh::config::{Config, ValidatedMap};
use zenoh::prelude::sync::SyncResolve;
use zenoh::prelude::*;

mod addresses;
use addresses::*;

#[derive(Deserialize, PartialEq, Default)]
struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Deserialize, PartialEq, Default)]
struct Twist {
    linear: Vector3,
    angular: Vector3,
}

fn main() {
    let args = App::new("zenoh turtlebot3 example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE]  'The zenoh session mode (peer by default).")
                .possible_values(["peer", "client"]),
        )
        .arg(
            Arg::from_usage("-e, --endpoints=[FILE]   'A BSSID/endpoint mapping json file.'")
                .required(true),
        )
        .arg(Arg::from_usage(
            "-l, --listen=[ENDPOINT]...   'Endpoints to listen on.'",
        ))
        .arg(
            Arg::from_usage("-p, --prefix=[KEYEXPR] 'The robot prefix'")
                .default_value("rt/turtle1"),
        )
        .arg(Arg::from_usage(
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .arg(
            Arg::from_usage("-s, --serial=[FILE]      'A serial port.'")
                .default_value("/dev/ttyACM0"),
        )
        .arg(
            Arg::from_usage(
                "-r, --resolution=[RESOLUTION] 'The resolution of the published video.",
            )
            .default_value("640x360"),
        )
        .arg(
            Arg::from_usage(
                "-q, --quality=[QUALITY] 'The quality of the published frames (0 - 100).",
            )
            .default_value("95"),
        )
        .arg(
            Arg::from_usage("-d, --delay=[TIME] 'The delay between each iteration in seconds.")
                .default_value("0.05"),
        )
        .arg(Arg::from_usage(
            "--no-multicast-scouting 'Disable the multicast-based scouting mechanism.'",
        ))
        .get_matches();

    let endpoints: Value = serde_json::from_str(
        &std::fs::read_to_string(args.value_of("endpoints").unwrap()).unwrap(),
    )
    .unwrap();

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

    let prefix = args.value_of("prefix").unwrap().to_string();
    let serial = args.value_of("serial").unwrap().to_string();
    let delay = (args.value_of("delay").unwrap().parse::<f32>().unwrap() * 1000.0) as u64;
    let resolution = args
        .value_of("resolution")
        .unwrap()
        .split('x')
        .map(|s| s.parse::<i32>().unwrap())
        .collect::<Vec<i32>>();
    let quality: i32 = args.value_of("quality").unwrap().parse().unwrap();

    let mut bssid = get_bssid().unwrap_or_else(|| "default".to_string());
    update_config(&mut config, &endpoints, &bssid);

    println!("[INFO] Open zenoh session...");
    let session = zenoh::open(config).res().unwrap();

    let heartbeat_pubsliher = session
        .declare_publisher(format!("{}/heartbeat", prefix))
        .res()
        .unwrap();

    println!("[INFO] Connect to motor...");
    let mut cmd = match dynamixel2::Bus::open(serial, 115200, Duration::from_secs(3)) {
        Ok(mut bus) => {
            let _ = bus.write_u8(200, IMU_RE_CALIBRATION, 1);
            Some((
                bus,
                session
                    .declare_subscriber(&format!("{}/cmd_vel", prefix))
                    .res()
                    .unwrap(),
            ))
        }
        Err(e) => {
            println!("[WARN] Unable to connect to motor: {}", e);
            None
        }
    };

    println!("[INFO] Open camera...");
    let mut camera = {
        let camer_publisher = session
            .declare_publisher(format!("{}/cams/0", prefix))
            .res()
            .unwrap();
        #[cfg(feature = "opencv-32")]
        let cam = videoio::VideoCapture::new_default(0).unwrap();
        #[cfg(not(feature = "opencv-32"))]
        let cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();
        if videoio::VideoCapture::is_opened(&cam).is_ok() {
            let mut encode_options = opencv::types::VectorOfi32::new();
            encode_options.push(opencv::imgcodecs::IMWRITE_JPEG_QUALITY);
            encode_options.push(quality);
            Some((cam, encode_options, camer_publisher))
        } else {
            println!("[WARN] Unable to open camera!");
            None
        }
    };

    sleep(Duration::from_secs(3));

    let mut count = 0;

    println!("[INFO] Running!");
    loop {
        if let Some((bus, sub)) = &mut cmd {
            let mut twist = Twist::default();
            while let Ok(recv) = sub.try_recv() {
                twist = cdr::deserialize::<Twist>(&recv.value.payload.contiguous()).unwrap();
            }

            let _ = bus.write_u8(200, HEARTBEAT, count);
            let _ = bus.write_u32(200, CMD_VELOCITY_LINEAR_X, (twist.linear.x as i32) as u32);
            let _ = bus.write_u32(200, CMD_VELOCITY_LINEAR_Y, (twist.linear.y as u32) as u32);
            let _ = bus.write_u32(200, CMD_VELOCITY_LINEAR_Z, (twist.linear.z as u32) as u32);
            let _ = bus.write_u32(200, CMD_VELOCITY_ANGULAR_X, (twist.angular.x as u32) as u32);
            let _ = bus.write_u32(200, CMD_VELOCITY_ANGULAR_Y, (twist.angular.y as u32) as u32);
            let _ = bus.write_u32(200, CMD_VELOCITY_ANGULAR_Z, (twist.angular.z as i32) as u32);
        }

        heartbeat_pubsliher.put(count as i64).res().unwrap();

        if let Some((cam, encode_options, cam_pub)) = &mut camera {
            let mut frame = core::Mat::default();
            cam.read(&mut frame).unwrap();

            let mut reduced = Mat::default();
            opencv::imgproc::resize(
                &frame,
                &mut reduced,
                opencv::core::Size::new(resolution[0], resolution[1]),
                0.0,
                0.0,
                opencv::imgproc::INTER_LINEAR,
            )
            .unwrap();
            let mut buf = opencv::types::VectorOfu8::new();
            opencv::imgcodecs::imencode(".jpeg", &reduced, &mut buf, encode_options).unwrap();
            cam_pub.put(buf.to_vec()).res().unwrap();
        }

        let new_bssid = get_bssid().unwrap_or_else(|| "default".to_string());
        if bssid != new_bssid {
            println!("[info] New access point detected");
            update_config(&mut session.config(), &endpoints, &new_bssid);
            bssid = new_bssid;
        }

        count = count.wrapping_add(1);
        sleep(Duration::from_millis(delay));
    }
}

fn update_config<T: ValidatedMap>(config: &mut T, mapping: &Value, new_bssid: &str) {
    if let Some(endpoints) = mapping
        .as_object()
        .unwrap()
        .get(new_bssid)
        .or_else(|| mapping.as_object().unwrap().get("default"))
    {
        config
            .insert_json5("connect/endpoints", endpoints.as_str().unwrap())
            .unwrap()
    }
}

fn get_bssid() -> Option<String> {
    std::process::Command::new("iwconfig")
        .output()
        .ok()
        .and_then(|output| {
            String::from_utf8_lossy(&output.stdout)
                .split('\n')
                .find(|line| line.contains("Access Point: "))
                .map(|s| {
                    s.split("Access Point: ")
                        .last()
                        .unwrap()
                        .split(' ')
                        .next()
                        .unwrap()
                        .to_string()
                })
        })
}

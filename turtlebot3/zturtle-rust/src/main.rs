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
use clap::Parser;
use opencv::{core, prelude::*, videoio};
use serde_derive::Deserialize;
use serde_json::Value;
use std::thread::sleep;
use std::time::Duration;
use zenoh::{Config, Wait};

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

#[derive(Parser, Debug)]
#[command(about = "Zenoh TurtleBot3 teleoperation")]
struct Args {
    #[arg(short, long)]
    mode: Option<String>,
    #[arg(short = 'e', long, required = true)]
    endpoints: String,
    #[arg(short, long)]
    listen: Vec<String>,
    #[arg(short, long, default_value = "rt/turtle1")]
    prefix: String,
    #[arg(short, long)]
    config: Option<String>,
    #[arg(short, long, default_value = "/dev/ttyACM0")]
    serial: String,
    #[arg(short, long, default_value = "640x360")]
    resolution: String,
    #[arg(short, long, default_value = "95")]
    quality: i32,
    #[arg(short, long, default_value = "0.05")]
    delay: f32,
    #[arg(long)]
    no_multicast_scouting: bool,
}

fn main() {
    let args = Args::parse();

    let endpoints: Value =
        serde_json::from_str(&std::fs::read_to_string(&args.endpoints).unwrap()).unwrap();

    let prefix = args.prefix.clone();
    let serial = args.serial.clone();
    let delay = (args.delay * 1000.0) as u64;
    let resolution = args
        .resolution
        .split('x')
        .map(|s| s.parse::<i32>().unwrap())
        .collect::<Vec<i32>>();
    let quality = args.quality;

    let mut bssid = get_bssid().unwrap_or_else(|| "default".to_string());
    let mut config = build_config(&args);
    update_config(&mut config, &endpoints, &bssid);

    println!("[INFO] Open zenoh session...");
    let session = zenoh::open(config).wait().unwrap();

    let heartbeat_publisher = session
        .declare_publisher(format!("{}/heartbeat", prefix))
        .wait()
        .unwrap();

    println!("[INFO] Connect to motor...");
    let mut cmd = match dynamixel2::Bus::open(&serial, 115200, Duration::from_secs(3)) {
        Ok(mut bus) => {
            let _ = bus.write_u8(200, IMU_RE_CALIBRATION, 1);
            Some((
                bus,
                session
                    .declare_subscriber(format!("{}/cmd_vel", prefix))
                    .wait()
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
        let cam_publisher = session
            .declare_publisher(format!("{}/cams/0", prefix))
            .wait()
            .unwrap();
        let cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();
        if videoio::VideoCapture::is_opened(&cam).unwrap_or(false) {
            let mut encode_options = opencv::core::Vector::<i32>::new();
            encode_options.push(opencv::imgcodecs::IMWRITE_JPEG_QUALITY);
            encode_options.push(quality);
            Some((cam, encode_options, cam_publisher))
        } else {
            println!("[WARN] Unable to open camera!");
            None
        }
    };

    sleep(Duration::from_secs(3));
    let mut count: u8 = 0;

    println!("[INFO] Running!");
    loop {
        if let Some((bus, sub)) = &mut cmd {
            let mut twist = Twist::default();
            while let Ok(recv) = sub.try_recv() {
                twist = cdr::deserialize::<Twist>(&recv.payload().to_bytes()).unwrap();
            }

            let _ = bus.write_u8(200, HEARTBEAT, count);
            let _ = bus.write_u32(200, CMD_VELOCITY_LINEAR_X, (twist.linear.x as i32) as u32);
            let _ = bus.write_u32(200, CMD_VELOCITY_LINEAR_Y, (twist.linear.y as u32) as u32);
            let _ = bus.write_u32(200, CMD_VELOCITY_LINEAR_Z, (twist.linear.z as u32) as u32);
            let _ = bus.write_u32(200, CMD_VELOCITY_ANGULAR_X, (twist.angular.x as u32) as u32);
            let _ = bus.write_u32(200, CMD_VELOCITY_ANGULAR_Y, (twist.angular.y as u32) as u32);
            let _ = bus.write_u32(200, CMD_VELOCITY_ANGULAR_Z, (twist.angular.z as i32) as u32);
        }

        heartbeat_publisher.put(count as i64).wait().unwrap();

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
            let mut buf = opencv::core::Vector::<u8>::new();
            opencv::imgcodecs::imencode(".jpeg", &reduced, &mut buf, encode_options).unwrap();
            cam_pub.put(buf.to_vec()).wait().unwrap();
        }

        let new_bssid = get_bssid().unwrap_or_else(|| "default".to_string());
        if bssid != new_bssid {
            println!("[INFO] New access point detected");
            let mut runtime_config = session.config().lock();
            update_config(&mut *runtime_config, &endpoints, &new_bssid);
            bssid = new_bssid;
        }

        count = count.wrapping_add(1);
        sleep(Duration::from_millis(delay));
    }
}

fn build_config(args: &Args) -> Config {
    use serde_json::json;
    let mut config = match &args.config {
        Some(path) => Config::from_file(path).unwrap(),
        None => Config::default(),
    };
    if let Some(mode) = &args.mode {
        config.insert_json5("mode", &json!(mode).to_string()).unwrap();
    }
    if !args.listen.is_empty() {
        config
            .insert_json5("listen/endpoints", &json!(args.listen).to_string())
            .unwrap();
    }
    if args.no_multicast_scouting {
        config
            .insert_json5("scouting/multicast/enabled", "false")
            .unwrap();
    }
    config
}

fn update_config(config: &mut zenoh::config::Config, mapping: &Value, new_bssid: &str) {
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

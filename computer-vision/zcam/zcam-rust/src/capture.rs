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
use futures::{select, FutureExt};
use opencv::{prelude::*, videoio};
use serde_json::json;
use zenoh::{config::Config, Wait};

#[tokio::main]
async fn main() {
    // initiate logging
    env_logger::init();

    let (config, key_expr, resolution, 
        delay, reliability, congestion_ctrl, image_quality) = parse_args();

    println!("Opening session...");
    let z = zenoh::open(config).await.unwrap();

    let publ = z.declare_publisher(&key_expr)
        .reliability(reliability)
        .congestion_control(congestion_ctrl)
        .await.unwrap();

    let conf_sub = z.declare_subscriber(format!("{}/zcapture/conf/**", key_expr)).await.unwrap();                

    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();

    let opened = videoio::VideoCapture::is_opened(&cam).unwrap();
    
    if !opened {
        panic!("Unable to open default camera!");
    }
    let mut encode_options = opencv::core::Vector::<i32>::new();    
    encode_options.push(opencv::imgcodecs::IMWRITE_JPEG_QUALITY);
    encode_options.push(image_quality);

    loop {
        select!(
            _ = tokio::time::sleep(std::time::Duration::from_millis(delay)).fuse() => {
                let mut frame = Mat::default();
                cam.read(&mut frame).unwrap();

                if !frame.empty() {
                    let mut reduced = Mat::default();
                    opencv::imgproc::resize(&frame, &mut reduced, opencv::core::Size::new(resolution[0], resolution[1]), 0.0, 0.0 , opencv::imgproc::INTER_LINEAR).unwrap();

                    let mut buf = opencv::core::Vector::<u8>::new();
                    opencv::imgcodecs::imencode(".jpeg", &reduced, &mut buf, &encode_options).unwrap();

                    publ.put(buf.to_vec()).wait().unwrap();
                } else {
                    println!("Reading empty buffer from camera... Waiting some more....");
                }
            },

            sample = conf_sub.recv_async().fuse() => {
                let sample = sample.unwrap();
                let conf_key = sample.key_expr().as_str().split("/conf/").last().unwrap();
                let conf_val = String::from_utf8_lossy(&sample.payload().to_bytes()).to_string();
                let _ = z.config().insert_json5(conf_key, &conf_val);
            },
        );
    }
}

#[derive(clap::Parser, Clone, PartialEq, Eq, Hash, Debug)]
struct Args {
    #[arg(short, long)]
    mode: Option<String>,
    
    #[arg(short, long, default_value="demo/zcam")]
    key: String,
    
    #[arg(short('e'), long)]
    connect: Option<Vec<String>>,
    
    #[arg(short, long)]
    config: Option<String>,

    #[arg(short, long, default_value="640x360")]
    resolution: String,

    #[arg(short, long, default_value="40")]
    delay: u64,

    #[arg(long, default_value="false")]
    best_effort: bool,

    #[arg(long, default_value="false")]
    block_on_congestion: bool,

    #[arg(long, default_value="18")]
    image_quality: i32,

}

fn parse_args() -> (Config, String, Vec<i32>, u64, zenoh::qos::Reliability, zenoh::qos::CongestionControl, i32) {
    let args = Args::parse();
    let mut c = 
        if let Some(f) = args.config { zenoh::Config::from_file(f).expect("Invalid Zenoh Configuraiton File") } 
        else { zenoh::Config::default() };

    if let Some(ls) = args.connect {                
        let _ = c.insert_json5("connect/endpoints", &json!(ls).to_string());        
    }
    if let Some(m) = args.mode {        
        let _ = c.insert_json5("mode", &json!(m).to_string());
    }

    let resolution = args.resolution        
        .split('x')
        .map(|s| s.parse::<i32>().unwrap())
        .collect::<Vec<i32>>();
        
    let congestion_control = 
        if args.block_on_congestion {zenoh::qos::CongestionControl::Block} else {zenoh::qos::CongestionControl::Drop};
    let reliability = if args.best_effort {zenoh::qos::Reliability::BestEffort} else { zenoh::qos::Reliability::Reliable };
    
    (c, args.key, resolution, args.delay, reliability, congestion_control, args.image_quality)
}

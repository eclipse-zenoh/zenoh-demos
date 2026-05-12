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
use opencv::{core::Scalar, imgproc, objdetect, prelude::*};
use serde_json::json;
use zcam::FrameMeta;
use zenoh::{config::Config, Wait};

#[tokio::main]
async fn main() {
    env_logger::init();

    // Load cascade
    let mut face_cascade = objdetect::CascadeClassifier::new(
        "haarcascade_frontalface_default.xml"
    ).unwrap();

    let (config, key_sub, key_pub, reliability, congestion_ctrl) = parse_args();

    println!("Opening session...");
    let z = zenoh::open(config).wait().unwrap();
    let sub = z.declare_subscriber(&key_sub).await.unwrap();

    let publ = z
        .declare_publisher(&key_pub)
        .reliability(reliability)
        .congestion_control(congestion_ctrl)
        .await
        .unwrap();

    let conf_sub = z
        .declare_subscriber(format!("{}/zdisplay/conf/**", key_sub))
        .wait()
        .unwrap();

    let sample = sub.recv_async().fuse().await;
    let sample = sample.unwrap();

    let meta = FrameMeta::decode(&sample.attachment().unwrap().to_bytes()).unwrap();


    loop {
        select!(
            sample = sub.recv_async().fuse() => {
                let sample = sample.unwrap();
                let bs = opencv::core::Vector::<u8>::from(sample.payload().to_bytes().to_vec());
                let mut decoded = opencv::imgcodecs::imdecode(
                    &bs,
                    opencv::imgcodecs::IMREAD_COLOR,
                ).unwrap();

                // Detect directly on the color Mat – no cvt_color needed
                let mut faces = opencv::core::Vector::new();
                face_cascade.detect_multi_scale(
                    &decoded,                           // color image here
                    &mut faces,
                    1.1,
                    3,
                    objdetect::CASCADE_SCALE_IMAGE,
                    opencv::core::Size::new(30, 30),
                    opencv::core::Size::new(0, 0),
                ).unwrap();

                // Draw green rectangles around found faces
                for face in faces {
                    imgproc::rectangle(
                        &mut decoded,
                        face,
                        Scalar::new(0.0, 255.0, 0.0, 0.0), // BGR green
                        2,
                        imgproc::LINE_8,
                        0,
                    ).unwrap();
                }

                drop(sample)
            },

            sample = conf_sub.recv_async().fuse() => {
                let sample = sample.unwrap();
                let conf_key = sample.key_expr().as_str().split("/conf/").last().unwrap();
                let conf_val = String::from_utf8_lossy(&sample.payload().to_bytes()).to_string();
                let _ = z.config().insert_json5(conf_key, &conf_val);
            },
        );
    }
    conf_sub.undeclare().wait().unwrap();
    sub.undeclare().wait().unwrap();
    z.close().wait().unwrap();
}

#[derive(clap::Parser, Clone, PartialEq, Eq, Hash)]
struct Args {
    #[arg(short, long)]
    mode: Option<String>,

    #[arg(short, long, default_value = "demo/zcam")]
    key_sub: String,

    #[arg(short, long, default_value = "demo/zcam/1")]
    key_pub: String,

    #[arg(short('e'), long)]
    connect: Option<Vec<String>>,

    #[arg(short, long)]
    config: Option<String>,

    #[arg(long, default_value = "false")]
    best_effort: bool,

    #[arg(long, default_value = "false")]
    block_on_congestion: bool,

}

fn parse_args() -> (Config, String, String,     zenoh::qos::Reliability,
    zenoh::qos::CongestionControl,
) {
    let args = Args::parse();
    let mut c = if let Some(f) = args.config {
        zenoh::Config::from_file(f).expect("Invalid Zenoh Configuraiton File")
    } else {
        zenoh::Config::default()
    };

    if let Some(ls) = args.connect {
        let _ = c.insert_json5("connect/endpoints", &json!(ls).to_string());
    }
    if let Some(m) = args.mode {
        let _ = c.insert_json5("mode", &json!(m).to_string());
    }

    let congestion_control = if args.block_on_congestion {
        zenoh::qos::CongestionControl::Block
    } else {
        zenoh::qos::CongestionControl::Drop
    };
    let reliability = if args.best_effort {
        zenoh::qos::Reliability::BestEffort
    } else {
        zenoh::qos::Reliability::Reliable
    };

    (c, args.key_sub, args.key_pub, 
        reliability,
        congestion_control)
}

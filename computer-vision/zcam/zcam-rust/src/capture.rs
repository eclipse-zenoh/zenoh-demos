//
// Copyright (c) 2017, 2020 ZettaScale Technology
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
//
// Contributors:
//   The Zenoh Team, <zenoh@zettascale.tech>
//
use clap::Parser;
use opencv::{prelude::*, videoio};
use rkyv::rancor::Error;
use serde_json::json;
use tokio::select;
use zcam::{config_update_loop, FrameMeta, RawFrameMeta};
use zenoh::{
    config::Config,
    qos::{CongestionControl, Reliability},
    shm::*,
    Session,
};

#[tokio::main]
async fn main() {
    // Initiate logging
    zenoh::init_log_from_env_or("error");

    // Parse command line arguments
    let (config, key_pub, delay, reliability, congestion_ctrl) = parse_args();

    println!("Opening session...");
    let z = zenoh::open(config).await.unwrap();

    select!(
        // Processing loop
        _ = process_loop(&z, key_pub.clone(), delay, reliability, congestion_ctrl) => {}
        // Config update loop
        _ = config_update_loop(&z, format!("{}/zcapture/conf/**", key_pub)) => {},
    );
}

#[derive(clap::Parser, Clone, PartialEq, Eq, Hash, Debug)]
struct Args {
    #[arg(short, long)]
    mode: Option<String>,

    #[arg(short, long, default_value = "demo/zcam")]
    key: String,

    #[arg(short('e'), long)]
    connect: Option<Vec<String>>,

    #[arg(short, long)]
    config: Option<String>,

    #[arg(short, long, default_value = "40")]
    delay: u64,

    #[arg(long, default_value = "false")]
    best_effort: bool,

    #[arg(long, default_value = "false")]
    block_on_congestion: bool,
}

fn parse_args() -> (
    Config,
    String,
    u64,
    zenoh::qos::Reliability,
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

    (c, args.key, args.delay, reliability, congestion_control)
}

/// Processing loop that captures frames from camera directly into Zenoh SHM and publishes them without copying.
async fn process_loop(
    session: &Session,
    key_pub: String,
    delay: u64,
    reliability: Reliability,
    congestion_ctrl: CongestionControl,
) {
    // Declare publisher for camera frames
    let publ = session
        .declare_publisher(key_pub)
        .reliability(reliability)
        .congestion_control(congestion_ctrl)
        .await
        .unwrap();

    // Open camera
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();
    if !videoio::VideoCapture::is_opened(&cam).unwrap() {
        panic!("Unable to open default camera!");
    }

    // Query the layout of the frame to use for SHM buffers allocation
    let (raw_meta, meta, encoded_meta) = {
        let mut frame = Mat::default();
        cam.read(&mut frame)
            .expect("Failed to read camera frame for detecting layout!");

        if frame.empty() {
            panic!("Camera returned empty frame!");
        }

        let raw_meta = RawFrameMeta::new(&frame).unwrap();
        let meta = FrameMeta::Raw(raw_meta.clone());
        let encoded_meta = rkyv::to_bytes::<Error>(&meta).unwrap().to_vec();

        (raw_meta, meta, encoded_meta)
    };

    // Obtain SHM provider from the session to allocate SHM buffers for frames
    let shm_provider = session
        .get_shm_provider()
        .await
        .expect("Failed to get transport SHM provider");

    tracing::info!("Will publish frames: {meta}...");
    loop {
        // Allocate SHM buffer for decoded frames with layout that is taken from the frame metadata
        let mut shm_buf = shm_provider
            .alloc(raw_meta.size())
            .with_policy::<BlockOn<GarbageCollect>>()
            .await
            .expect("Failed to allocate SHM buffer");

        // Map opencv Mat into shared memory
        let mut frame = unsafe { raw_meta.mat_mut(shm_buf.as_mut_ptr()) };

        // Capture frame directly into SHM buffer using shm-backed Mat
        cam.read(&mut frame).expect("Failed to read camera frame!");

        if !frame.empty() {
            // Publish the frame with the encoded meta as attachment
            publ.put(shm_buf)
                .attachment(&encoded_meta)
                .await
                .expect("Failed to publish camera frame!");
        } else {
            tracing::error!("Reading empty buffer from camera... Waiting some more....");
        }

        // Wait before capturing next frame to maintin the desired frame rate
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }
}

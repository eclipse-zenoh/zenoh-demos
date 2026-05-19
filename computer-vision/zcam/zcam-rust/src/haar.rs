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
use serde_json::json;

use futures::StreamExt;
use opencv::{
    core::{Scalar, ToInputArray, ToInputOutputArray},
    imgproc, objdetect,
    prelude::*,
};
use std::convert::TryInto;
use tokio::select;
use zcam::FrameMeta;
use zenoh::{config::Config, sample::Sample, shm::zshmmut};

#[tokio::main]
async fn main() {
    // Initiate logging
    zenoh::init_log_from_env_or("error");

    // Parse command line arguments
    let (haarcascade_file, config, key_sub, key_pub, reliability, congestion_ctrl) = parse_args();

    // Load cascade
    let mut cascade = objdetect::CascadeClassifier::new(&haarcascade_file).unwrap();

    println!("Opening session...");
    let z = zenoh::open(config).await.unwrap();

    // Declare subscriber for frames
    let sub = z.declare_subscriber(&key_sub).await.unwrap();

    // Declare publisher for processed frames
    let publ = z
        .declare_publisher(&key_pub)
        .reliability(reliability)
        .congestion_control(congestion_ctrl)
        .await
        .unwrap();

    // Declare subscriber for configuration updates
    let conf_sub = z
        .declare_subscriber(format!("{}/zhaar/conf/**", key_sub))
        .await
        .unwrap();

    select!(
        stream_result =  sub
            .stream()
            .map(|mut sample| -> zenoh::Result<Sample> {
                // Map the incoming sample into a _mutable_ OpenCV Mat, trying to do it in-place via SHM when possible,
                // and falling back to copying if any of the SHM steps fail
                sample_to_frame(&mut sample).and_then(|mut frame| {
                    detect_objects(&mut frame, &mut cascade)?;
                    Ok(sample)
                })
            })
            // Forward the processed samples to the publisher, ensuring that any errors in the stream are propagated
            // NOTE: depening on the initial sample structure (SHM, SHM mutable, or non-SHM),
            // each forwarded sample may involve
            // 1. zero-copy, if the original sample was SHM mutable: we in-place mutate it and republish without copying
            // 2. one copy if scenario 1 fails
            // Additionally, for option 2 sample still may be republished as SHM due to zenoh's ability to transparently
            // convert non-SHM payloads ("implicit SHM optimization") into SHM ones when applicable
            .forward(publ) => { stream_result.unwrap(); }

        _ = async {
            let sample = conf_sub.recv_async().await.unwrap();
            let conf_key = sample.key_expr().as_str().split("/conf/").last().unwrap();
            let conf_val = String::from_utf8_lossy(&sample.payload().to_bytes()).to_string();
            let _ = z.config().insert_json5(conf_key, &conf_val);
         } => {},
    );
}

#[derive(clap::Parser, Clone, PartialEq, Eq, Hash)]
struct Args {
    #[arg(long, default_value = "haarcascade_frontalface_default.xml")]
    haarcascade_file: String,

    #[arg(short, long)]
    mode: Option<String>,

    #[arg(short, long, default_value = "demo/zcam")]
    key_sub: String,

    #[arg(long, default_value = "demo/zcam/haarcascade")]
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

fn parse_args() -> (
    String,
    Config,
    String,
    String,
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

    (
        args.haarcascade_file,
        c,
        args.key_sub,
        args.key_pub,
        reliability,
        congestion_control,
    )
}

fn sample_to_frame(
    sample: &mut Sample,
) -> zenoh::Result<impl MatTraitConst + ToInputArray + ToInputOutputArray> {
    let meta = FrameMeta::decode_from_sample(&sample)?;
    if let FrameMeta::Raw(raw_meta) = meta {
        fn try_shm_inplace(
            raw_meta: &zcam::RawFrameMeta,
            sample: &mut Sample,
        ) -> zenoh::Result<Mat> {
            // Try to interpret the payload as SHM buffer
            let shm_buf = sample
                .payload_mut()
                .as_shm_mut()
                .ok_or("Missing SHM buffer")?;

            // Try to get mutable access to the SHM buffer
            let shm_buf_mut: &mut zshmmut = shm_buf
                .try_into()
                .map_err(|_| "Unable to mutate SHM data")?;

            // Interpret the SHM buffer as OpenCV Mat
            let frame = unsafe { raw_meta.mat_mut(shm_buf_mut.as_mut_ptr()) };

            Ok(frame)
        }

        // First try to interpret the payload as SHM buffer and map it directly without copying
        try_shm_inplace(&raw_meta, sample).or_else(|err| {
            // If any of the SHM steps fail, fall back to copying the data into a new buffer
            tracing::debug!("SHM inplace failed: {err}. Falling back to copy...");

            // Copy the payload bytes into a new contiguous buffer
            let mut contiguous_bytes = sample.payload().to_bytes().to_vec();

            // Interpret the new buffer as OpenCV Mat
            let frame = unsafe { raw_meta.mat_mut(contiguous_bytes.as_mut_ptr()) };

            // Update the sample payload to point to the new contiguous buffer
            sample.set_payload(contiguous_bytes.into());

            Ok(frame)
        })
    } else {
        let err = format!("Unsupported frame meta: {:?}", meta);
        eprintln!("{err}");
        Err(err.into())
    }
}

fn detect_objects<F: MatTraitConst + ToInputArray + ToInputOutputArray>(
    frame: &mut F,
    cascade: &mut objdetect::CascadeClassifier,
) -> opencv::Result<()> {
    // Detect directly on the color Mat – no cvt_color needed
    let mut objects = opencv::core::Vector::new();
    cascade.detect_multi_scale(
        frame, // color image here
        &mut objects,
        1.1,
        3,
        objdetect::CASCADE_SCALE_IMAGE,
        opencv::core::Size::new(30, 30),
        opencv::core::Size::new(0, 0),
    )?;

    // Draw green rectangles around found objects
    for object in objects {
        imgproc::rectangle(
            frame,
            object,
            Scalar::new(0.0, 255.0, 0.0, 0.0), // BGR green
            2,
            imgproc::LINE_8,
            0,
        )
        .unwrap();
    }
    Ok(())
}

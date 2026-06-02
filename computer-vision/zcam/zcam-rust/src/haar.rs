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

use opencv::{
    core::{Scalar, ToInputArray, ToInputOutputArray},
    imgproc,
    objdetect::{self, CascadeClassifier},
    prelude::*,
};
use std::{convert::TryInto, io::Read, sync::Arc};
use tokio::select;
use zcam::{config_update_loop, FrameMeta};
use zenoh::{
    config::Config,
    qos::{CongestionControl, Reliability},
    sample::Sample,
    shm::{
        zshm, zshmmut, BlockOn, Defragment, GarbageCollect, PosixShmProviderBackend, ShmProvider,
        ZShm,
    },
    Session,
};

#[tokio::main]
async fn main() {
    // Initiate logging
    zenoh::init_log_from_env_or("error");

    // Parse command line arguments
    let (haarcascade_file, config, key_sub, key_pub, reliability, congestion_ctrl) = parse_args();

    // Load cascade
    let cascade = objdetect::CascadeClassifier::new(&haarcascade_file).unwrap();

    println!("Opening session...");
    let z = zenoh::open(config).await.unwrap();

    select!(
        // Processing loop
        _ = process_loop(&z,key_sub.clone(), key_pub, reliability, congestion_ctrl, cascade) => {}
        // Config update loop
        _ = config_update_loop(&z, format!("{}/zhaar/conf/**", key_sub)) => {},
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

/// Processing loop that subscribes to frames, detects objects using Haar cascades and republishes the processed frames.
/// Frames are processed in-place directly in SHM without copying whenever possible, leveraging Zenoh's zero-copy accessors
/// and metadata information to map OpenCV Mats into SHM buffers. This makes the processing efficient even for large frames,
/// as it avoids unnecessary copying of frame data. If in-place processing is not possible for any reason, the code falls back
/// to copying the frame into a new SHM buffer for processing and republishing, ensuring that the processing is robust in all cases.
async fn process_loop(
    session: &Session,
    key_sub: String,
    key_pub: String,
    reliability: Reliability,
    congestion_ctrl: CongestionControl,
    mut cascade: CascadeClassifier,
) {
    // Declare subscriber for frames
    let sub = session.declare_subscriber(&key_sub).await.unwrap();

    // Declare publisher for processed frames
    let publ = session
        .declare_publisher(&key_pub)
        .reliability(reliability)
        .congestion_control(congestion_ctrl)
        .await
        .unwrap();

    // Obtain SHM provider from the session to allocate SHM buffers for frames
    let shm_provider = session
        .get_shm_provider()
        .await
        .expect("Failed to get transport SHM provider");

    loop {
        // Receive sample with frame
        let mut sample = sub.recv_async().await.unwrap();

        // Prcess the recieved frame and obtain the processed frame in SHM
        if let Ok(processed_frame_in_shm) =
            process_frame(&mut sample, &shm_provider, &mut cascade).await
        {
            // Publish SHM frame
            // NOTE:
            //      - Will be published as SHM payload for SHM-compatible subscribers
            //      - will be published as Raw payload in other cases
            publ.put(processed_frame_in_shm)
                .attachment(sample.attachment().cloned())
                .await
                .unwrap();
        }
    }
}

async fn process_frame(
    sample: &mut Sample,
    shm_provider: &Arc<ShmProvider<PosixShmProviderBackend>>,
    cascade: &mut objdetect::CascadeClassifier,
) -> zenoh::Result<ZShm> {
    // Decode frame metadata
    let meta = FrameMeta::decode(sample)
        .expect("Unable to decode frame metadata: probably wrong data format");

    match meta {
        FrameMeta::Raw(raw_meta) => {
            fn try_mutate_shm_inplace(sample: &mut Sample) -> Option<&mut zshmmut> {
                // Try to interpret the payload as SHM buffer
                let shm_buf = sample.payload_mut().as_shm_mut()?;

                // Try to get mutable access to the SHM buffer
                shm_buf.try_into().ok()
            }

            // First, try to process the frame in-place without copying if the payload is already an SHM buffer
            // and if we can get mutable access to it. This is the most efficient path as it avoids any copying.
            match try_mutate_shm_inplace(sample) {
                Some(shm_mut_inplace) => {
                    // Map opencv Mat into shared memory
                    let mut frame = unsafe { raw_meta.mat_mut(shm_mut_inplace.as_mut_ptr()) };

                    // Detect objects and draw rectangles in-place directly on the SHM buffer
                    detect_objects(&mut frame, cascade)?;

                    let shm_immut: &mut zshm = shm_mut_inplace.into();

                    // Return the processed frame as SHM without copying
                    Ok(shm_immut.to_owned())
                }
                None => {
                    // If any of the in-place SHM mutation steps fail, fall back to copying the data into a new SHM buffer
                    tracing::debug!("SHM inplace failed, falling back to copy...");

                    let payload = sample.payload();

                    // Allocate SHM buffer for contiguous payload bytes
                    let mut shmbuf: zenoh::shm::ZShmMut = unsafe {
                        shm_provider
                            .alloc(payload.len())
                            .with_unsafe_policy::<BlockOn<Defragment<GarbageCollect>>>()
                            .await
                            .expect("Failed to allocate SHM buffer")
                    };

                    // Read bytes directly into SHM buffer
                    payload.reader().read_exact(&mut shmbuf)?;

                    // Map opencv Mat into allocated shared memory
                    let mut frame = unsafe { raw_meta.mat_mut(shmbuf.as_mut_ptr()) };

                    // Detect objects and draw rectangles directly on the SHM buffer
                    detect_objects(&mut frame, cascade)?;

                    // Return the processed frame as SHM
                    Ok(shmbuf.into())
                }
            }
        }
        FrameMeta::Jpeg(_) => {
            let err = format!("Unsupported frame meta: {:?}", meta);
            tracing::error!("{err}");
            Err(err.into())
        }
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

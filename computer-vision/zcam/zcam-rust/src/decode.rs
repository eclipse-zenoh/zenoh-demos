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
use std::ops::Deref;

use futures::StreamExt;
use tokio::select;
use zcam::FrameMeta;
use zenoh::{config::Config, sample::Sample, shm::*, Wait};

#[tokio::main]
async fn main() {
    // Initiate logging.
    zenoh::init_log_from_env_or("error");

    // Parse command line arguments.
    let (config, key_sub, key_pub, reliability, congestion_ctrl) = parse_args();

    println!("Opening session...");
    let z = zenoh::open(config).wait().unwrap();

    // Declare subscriber for frames
    let sub = z.declare_subscriber(&key_sub).await.unwrap();

    // Declare publisher for decoded frames
    let publ = z
        .declare_publisher(&key_pub)
        .reliability(reliability)
        .congestion_control(congestion_ctrl)
        .await
        .unwrap();

    // Declare subscriber for configuration updates
    let conf_sub = z
        .declare_subscriber(format!("{}/zdecode/conf/**", key_sub))
        .wait()
        .unwrap();

    // )btain SHM provider from the session to allocate SHM buffers for frames.
    let shm_provider = z
        .get_shm_provider()
        .await
        .expect("Failed to get transport SHM provider");

    select!(
        stream_result =  sub
            .stream()
            .map(|mut sample| -> zenoh::Result<Sample> {
                // Decode frame into SHM.
                ensure_raw(&shm_provider, &mut sample,).and_then(|_| {
                    Ok(sample)
                })
            })
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
    #[arg(short, long)]
    mode: Option<String>,

    #[arg(short, long, default_value = "demo/zcam/encoded")]
    key_sub: String,

    #[arg(long, default_value = "demo/zcam/decoded")]
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
        c,
        args.key_sub,
        args.key_pub,
        reliability,
        congestion_control,
    )
}

fn ensure_raw<Backend: ShmProviderBackend>(
    shm_provider: &ShmProvider<Backend>,
    sample: &mut Sample,
) -> zenoh::Result<()> {
    let meta = FrameMeta::decode_from_sample(sample)?;
    match meta {
        FrameMeta::Raw(_) => {}
        FrameMeta::Jpeg(raw_meta) => {
            // Allocate SHM buffer for decoded frames with layout that is taken from the frame metadata.
            let mut shmbuf = shm_provider
                .alloc(raw_meta.size())
                .with_policy::<BlockOn<Defragment<GarbageCollect>>>()
                .wait()
                .expect("Failed to allocate SHM buffer");

            // Map opencv Mat into shared memory
            let mut decoded_frame = unsafe { raw_meta.mat_mut(shmbuf.as_mut_ptr()) };

            // This Cow accessor provides immutable access to contained data.
            // Access will be zero-copy if data is contiguous.
            let contiguous_bytes = sample.payload().to_bytes();

            // Decode frame into SHM buffer using shm-backed Mat
            opencv::imgcodecs::imdecode_to(
                &contiguous_bytes.deref(),
                opencv::imgcodecs::IMREAD_COLOR,
                &mut decoded_frame,
            )?;

            // Replace sample metadata because now it is raw
            FrameMeta::Raw(raw_meta).encode_to_sample(sample)?;
            // Update sample payload.
            sample.set_payload(shmbuf.into());
        }
    }
    Ok(())
}

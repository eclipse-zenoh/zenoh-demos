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

use tokio::select;
use zcam::{config_update_loop, FrameMeta};
use zenoh::{
    config::Config,
    qos::{CongestionControl, Reliability},
    Session, Wait,
};

#[tokio::main]
async fn main() {
    // Initiate logging
    zenoh::init_log_from_env_or("error");

    // Parse command line arguments
    let (config, key_sub, key_pub, reliability, congestion_ctrl) = parse_args();

    println!("Opening session...");
    let z = zenoh::open(config).wait().unwrap();

    select!(
        // Processing loop
        _ = process_loop(&z,key_sub.clone(), key_pub, reliability, congestion_ctrl) => {}
        // Config update loop
        _ = config_update_loop(&z, format!("{}/zencode/conf/**", key_sub)) => {},
    );
}

#[derive(clap::Parser, Clone, PartialEq, Eq, Hash)]
struct Args {
    #[arg(short, long)]
    mode: Option<String>,

    #[arg(short, long, default_value = "demo/zcam")]
    key_sub: String,

    #[arg(long, default_value = "demo/zcam/encoded")]
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

/// Processing loop that subscribes to frames, encodes them and republishes encoded frames.
async fn process_loop(
    session: &Session,
    key_sub: String,
    key_pub: String,
    reliability: Reliability,
    congestion_ctrl: CongestionControl,
) {
    // Declare subscriber for frames
    let sub = session.declare_subscriber(&key_sub).await.unwrap();

    // Declare publisher for encoded frames
    let publ = session
        .declare_publisher(&key_pub)
        .reliability(reliability)
        .congestion_control(congestion_ctrl)
        .await
        .unwrap();

    // Prepare jpg encoder options
    let mut encode_options = opencv::core::Vector::<i32>::new();
    encode_options.push(opencv::imgcodecs::IMWRITE_JPEG_QUALITY);
    encode_options.push(90);

    loop {
        // Receive sample with frame
        let sample = sub.recv_async().await.unwrap();

        // Decode frame metadata
        let meta = FrameMeta::decode(&sample)
            .expect("Unable to decode frame metadata: probably wrong data format");

        match meta {
            FrameMeta::Raw(raw_meta) => {
                let jpeg_buf = {
                    // This Cow accessor provides immutable access to contained data.
                    // Access will be zero-copy if data is contiguous (including SHM case).
                    let contiguous_bytes = sample.payload().to_bytes();

                    // Map opencv Mat into contiguous payload bytes
                    let frame = unsafe { raw_meta.mat(contiguous_bytes.as_ptr()) };

                    // Encode as jpeg
                    let mut buf = opencv::core::Vector::<u8>::new();
                    opencv::imgcodecs::imencode(".jpeg", &frame, &mut buf, &encode_options)
                        .expect("Failed to encode frame to Jpeg");

                    buf
                };

                // Encode frame metadata
                let attachment = FrameMeta::Jpeg(raw_meta).encode().unwrap();

                // Publish encoded frame
                // NOTE:
                //      - may leverage Zenoh's implicit SHM optimization and be published as SHM payload
                //        for SHM-compatible subscribers
                //      - will be published as Raw payload in other cases
                publ.put(jpeg_buf.as_slice())
                    .attachment(attachment)
                    .await
                    .unwrap();
            }
            FrameMeta::Jpeg(_) => {
                // Already encoded - republish sample as it is
                // NOTE: depending on initial sample's payload, the following options available:
                // 1, SHM payload:
                //      - will be published as 100% zerocopy SHM payload for SHM-compatible subscribers
                //      - will be published as Raw payload in other cases
                // 2. Raw payload:
                //      - may leverage Zenoh's implicit SHM optimization and be published as SHM payload
                //        for SHM-compatible subscribers
                //      - will be published as Raw payload in other cases
                publ.put(sample.payload().to_owned())
                    .attachment(sample.attachment().cloned())
                    .await
                    .unwrap();
            }
        }
    }
}

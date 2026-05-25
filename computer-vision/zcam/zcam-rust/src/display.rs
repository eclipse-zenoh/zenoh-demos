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
use futures::StreamExt;
use opencv::highgui;
use serde_json::json;
use tokio::select;
use zcam::{config_update_loop, FrameMeta};
use zenoh::{config::Config, Session, Wait};

#[tokio::main]
async fn main() {
    // Initiate logging
    zenoh::init_log_from_env_or("error");

    // Parse command line arguments
    let (config, key_sub) = parse_args();

    println!("Opening session...");
    let z = zenoh::open(config).wait().unwrap();

    select!(
        // Processing loop
        _ = process_loop(&z, key_sub.clone()) => {}
        // Config update loop
        _ = config_update_loop(&z, format!("{}/zdisplay/conf/**", key_sub)) => {},
    );
}

#[derive(clap::Parser, Clone, PartialEq, Eq, Hash)]
struct Args {
    #[arg(short, long)]
    mode: Option<String>,

    #[arg(short, long, default_value = "demo/zcam")]
    key: String,

    #[arg(short('e'), long)]
    connect: Option<Vec<String>>,

    #[arg(short, long)]
    config: Option<String>,
}

fn parse_args() -> (Config, String) {
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

    (c, args.key)
}

/// Processing loop that subscribes to frames and displays them using OpenCV's highgui module.
/// Frames are mapped into OpenCV Mats without copying using metadata information and Zenoh's
/// zero-copy accessors, so the processing is efficient even for large frames.
async fn process_loop(session: &Session, key_sub: String) {
    // Declare subscriber for frames
    let sub = session.declare_subscriber(key_sub).await.unwrap();

    // Process incoming frames: read frames from stream and display them
    sub.stream()
        .for_each(async |sample| {
            // Decode frame metadata
            let meta = FrameMeta::decode(&sample)
                .expect("Unable to decode frame metadata: probably wrong data format");

            match meta {
                FrameMeta::Raw(raw_frame_meta) => {
                    // This Cow accessor provides immutable access to contained data.
                    // Access will be zero-copy if data is contiguous (including SHM case).
                    let contiguous_bytes = sample.payload().to_bytes();

                    // Map opencv Mat into contiguous payload bytes
                    let frame = unsafe { raw_frame_meta.mat(contiguous_bytes.as_ptr()) };

                    // Display the frame
                    highgui::imshow(sample.key_expr().as_str(), &frame)
                        .expect("Failed to display frame!");
                    if highgui::poll_key().unwrap() == 113 {
                        // 'q' key
                        std::process::exit(0);
                    }
                }
                other_meta => {
                    tracing::error!("Unsupported frame meta: {:?}", other_meta);
                }
            }
        })
        .await;
}

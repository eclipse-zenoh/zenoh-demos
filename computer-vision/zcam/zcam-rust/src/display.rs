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
use futures::{select, FutureExt, StreamExt};
use opencv::highgui;
use serde_json::json;
use zcam::FrameMeta;
use zenoh::{config::Config, sample::Sample, Wait};

#[tokio::main]
async fn main() {
    // Initiate logging
    zenoh::init_log_from_env_or("error");

    // Parse command line arguments
    let (config, key_expr) = parse_args();

    println!("Opening session...");
    let z = zenoh::open(config).wait().unwrap();

    // Declare subscriber for frames
    let sub = z.declare_subscriber(&key_expr).await.unwrap();

    // Declare subscriber for configuration updates
    let conf_sub = z
        .declare_subscriber(format!("{}/zdisplay/conf/**", key_expr))
        .wait()
        .unwrap();

    // Handle exit from GUI
    let _ = std::thread::spawn(|| {
        if highgui::wait_key(10).unwrap() == 113 {
            // 'q'
            std::process::exit(0);
        }
    });

    loop {
        select!(
            _ =  sub.stream().for_each(async |sample| {
                // Read frames from stream and display them
                if let Err(e) = display(&sample) {
                    tracing::error!("{e}");
                }
            }) => {},
            sample = conf_sub.recv_async().fuse() => {
                let sample = sample.unwrap();
                let conf_key = sample.key_expr().as_str().split("/conf/").last().unwrap();
                let conf_val = String::from_utf8_lossy(&sample.payload().to_bytes()).to_string();
                let _ = z.config().insert_json5(conf_key, &conf_val);
            },
        );
    }
}

#[derive(clap::Parser, Clone, PartialEq, Eq, Hash)]
struct Args {
    #[arg(short, long)]
    mode: Option<String>,

    #[arg(short, long, default_value = "demo/zcam/haarcascade")]
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

fn display(sample: &Sample) -> zenoh::Result<()> {
    let meta = FrameMeta::decode_from_sample(&sample)?;
    if let FrameMeta::Raw(raw_meta) = meta {
        // This Cow accessor provides immutable access to contained data.
        // Access will be zero-copy if data is contiguous.
        let contiguous_bytes = sample.payload().to_bytes();

        // Map opencv Mat into shared memory
        let frame = unsafe { raw_meta.mat(contiguous_bytes.as_ptr()) };

        // Display the frame directly from shared memory
        highgui::imshow(sample.key_expr().as_str(), &frame).unwrap();
        Ok(())
    } else {
        Err(format!("Unsupported frame meta: {:?}", meta).into())
    }
}

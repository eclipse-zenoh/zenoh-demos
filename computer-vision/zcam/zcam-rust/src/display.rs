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
use opencv::highgui;
use rkyv::rancor::Error;
use serde_json::json;
use zcam::{ArchivedFrameMeta, FrameMeta};
use zenoh::{config::Config, Wait};

#[tokio::main]
async fn main() {
    env_logger::init();
    let (config, key_expr) = parse_args();

    println!("Opening session...");
    let z = zenoh::open(config).wait().unwrap();
    let sub = z.declare_subscriber(&key_expr).await.unwrap();

    let conf_sub = z
        .declare_subscriber(format!("{}/zdisplay/conf/**", key_expr))
        .wait()
        .unwrap();

    loop {
        select!(
               sample = sub.recv_async().fuse() => {

               let sample = sample.unwrap();



               let attachment_bytes = sample.attachment().unwrap().to_bytes();
                let meta =  rkyv::access::<ArchivedFrameMeta, Error>(&attachment_bytes).unwrap();
        let meta = rkyv::deserialize:: <FrameMeta,Error>(meta).unwrap();



                   if let FrameMeta::Raw(raw_meta) = meta {
                       let shm_buf = sample.payload().as_shm().unwrap();
                       let frame = unsafe { raw_meta.mat(shm_buf.as_ptr()) };

                       highgui::imshow(sample.key_expr().as_str(), &frame).unwrap();
                       if highgui::wait_key(10).unwrap() == 113 {
                           // 'q'
                           break;
                       }
                       drop(sample)
                   } else {
                          eprintln!("Unsupported frame meta: {:?}", meta);
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
    conf_sub.undeclare().wait().unwrap();
    sub.undeclare().wait().unwrap();
    z.close().wait().unwrap();
}

#[derive(clap::Parser, Clone, PartialEq, Eq, Hash)]
struct Args {
    #[arg(short, long)]
    mode: Option<String>,

    #[arg(short, long, default_value = "demo/zcam/facedetect")]
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

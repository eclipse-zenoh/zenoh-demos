//
// Copyright (c) 2024 Atostek Oy
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   Juhana Helovuo <juhana.helovuo@atostek.com>
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
mod zenoh_examples_common;

use clap::Parser;
use std::time::Duration;
use zenoh::config::Config;
use zenoh_examples_common::CommonArgs;

use byteorder::{BigEndian, LittleEndian};
use cdr_encoding as cdr;
use serde::{Deserialize, Serialize};
use tokio::join;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct ShapeType {
    color: String,
    x: i32,
    y: i32,
    size: i32,
}

#[tokio::main]
async fn main() {
    zenoh::init_log_from_env_or("error");

    let Args {
        topic,
        color,
        common_args,
        do_publisher,
        do_subscriber,
    } = Args::parse();

    println!("Opening Zenoh session...");
    let config: Config = common_args.into();
    let session = zenoh::open(config).await.unwrap();

    println!("Press CTRL-C to quit...");

    let key_expr = topic.clone();

    if !(do_publisher || do_subscriber) {
        println!("Please specify --publisher or --subscriber to get anything done.");
        std::process::exit(-1);
    }

    join!(
        async {
            if do_publisher {
                println!("Declaring Publisher on '{key_expr}'...");
                let publisher = session.declare_publisher(&key_expr).await.unwrap();
                for idx in 2000..i32::MAX {
                    let shape = ShapeType {
                        color: color.clone(),
                        x: 20 + idx % 101,
                        y: 20 + idx % 123,
                        size: 32,
                    };
                    println!("Putting Data ('{}': {:?})...", &key_expr, &shape);
                    // SerializedPayload header: 0x00,0x01,0x00,0x00 = CDR Little-Endian
                    let mut payload_buffer = vec![0x00, 0x01, 0x00, 0x00];
                    payload_buffer
                        .append(&mut cdr::to_vec::<ShapeType, LittleEndian>(&shape).unwrap());
                    publisher.put(payload_buffer).await.unwrap();
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
            }
        },
        async {
            if do_subscriber {
                println!("Declaring Subscriber on '{key_expr}'...");
                let subscriber = session.declare_subscriber(&key_expr).await.unwrap();
                while let Ok(sample) = subscriber.recv_async().await {
                    let ser_payload = sample.payload().to_bytes().to_vec();
                    let shape = if ser_payload.len() < 4 {
                        Err("Too short SerializedPayload.".to_string())
                    } else {
                        let (id_and_opts, value) = ser_payload.split_at(4);
                        match id_and_opts[0..2] {
                            [0x00, 0x01] => {
                                Ok(cdr::from_bytes::<ShapeType, LittleEndian>(value).unwrap().0)
                            }
                            [0x00, 0x00] => {
                                Ok(cdr::from_bytes::<ShapeType, BigEndian>(value).unwrap().0)
                            }
                            ref r => Err(format!("Unknown RepresentationIdentifier {r:?}")),
                        }
                    };
                    println!(
                        "Received {} '{}': {:?}",
                        sample.kind(),
                        sample.key_expr().as_str(),
                        shape,
                    );
                }
            }
        }
    );
}

#[derive(clap::Parser, Clone, PartialEq, Eq, Hash, Debug)]
struct Args {
    #[arg(short, long, default_value = "Square")]
    /// Shape topic.
    topic: String,

    #[arg(long, default_value = "GREEN")]
    /// Shape default color.
    color: String,

    #[arg(id = "publisher", short = 'P', long)]
    /// Act as a publisher
    do_publisher: bool,

    #[arg(id = "subscriber", short = 'S', long)]
    /// Act as a subscriber
    do_subscriber: bool,

    #[command(flatten)]
    common_args: CommonArgs,
}

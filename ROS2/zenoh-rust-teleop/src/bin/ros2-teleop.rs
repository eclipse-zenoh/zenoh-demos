//
// Copyright (c) 2021 ZettaScale Technology
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
use cdr::{CdrLe, Infinite};
use clap::Parser;
use crossterm::{
    cursor::MoveToColumn,
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    ExecutableCommand,
};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use std::io::{stdout, Write};
use zenoh::Config;

#[derive(Serialize, PartialEq, Debug)]
struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Serialize, PartialEq, Debug)]
struct Twist {
    linear: Vector3,
    angular: Vector3,
}

#[derive(Deserialize, PartialEq)]
struct Time {
    sec: i32,
    nanosec: u32,
}

#[derive(Deserialize, PartialEq)]
struct Log {
    stamp: Time,
    level: u8,
    name: String,
    msg: String,
    file: String,
    function: String,
    line: u32,
}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}.{}] [{}]: {}",
            self.stamp.sec, self.stamp.nanosec, self.name, self.msg
        )
    }
}

async fn pub_twist(publisher: &zenoh::pubsub::Publisher<'_>, linear: f64, angular: f64) {
    let twist = Twist {
        linear: Vector3 { x: linear, y: 0.0, z: 0.0 },
        angular: Vector3 { x: 0.0, y: 0.0, z: angular },
    };
    write!(stdout(), "Publish on {} : {:?}\r\n", publisher.key_expr().as_str(), twist)
        .unwrap_or_default();
    let encoded = cdr::serialize::<_, _, CdrLe>(&twist, Infinite).unwrap();
    if let Err(e) = publisher.put(encoded).await {
        eprintln!("Error writing {}: {}", publisher.key_expr().as_str(), e);
    }
}

async fn del_twist(publisher: &zenoh::pubsub::Publisher<'_>) {
    write!(stdout(), "Delete on {}\r\n", publisher.key_expr().as_str()).unwrap_or_default();
    if let Err(e) = publisher.delete().await {
        eprintln!("Error deleting {}: {}", publisher.key_expr().as_str(), e);
    }
}

#[derive(Parser, Debug)]
#[command(about = "ROS2 keyboard teleop over Zenoh")]
struct Args {
    #[arg(short, long)]
    config: Option<String>,
    #[arg(long)]
    no_multicast_scouting: bool,
    #[arg(short, long)]
    mode: Option<String>,
    #[arg(short = 'e', long)]
    connect: Vec<String>,
    #[arg(short, long)]
    listen: Vec<String>,
    #[arg(long, default_value = "rt/turtle1/cmd_vel")]
    cmd_vel: String,
    #[arg(long, default_value = "rt/rosout")]
    rosout: String,
    #[arg(short = 'a', long, default_value = "2.0")]
    angular_scale: f64,
    #[arg(short = 'x', long, default_value = "2.0")]
    linear_scale: f64,
}

#[tokio::main]
async fn main() {
    zenoh::init_log_from_env_or("error");
    let args = Args::parse();
    let config = build_config(&args);
    let linear_scale = args.linear_scale;
    let angular_scale = args.angular_scale;

    println!("Opening session...");
    let session = zenoh::open(config).await.unwrap();

    println!("Subscriber on {}", args.rosout);
    let subscriber = session.declare_subscriber(&args.rosout).await.unwrap();
    let publisher = session.declare_publisher(&args.cmd_vel).await.unwrap();

    crossterm::terminal::enable_raw_mode().unwrap();
    let (key_sender, mut key_receiver) = tokio::sync::mpsc::channel::<Event>(10);
    tokio::spawn(async move {
        loop {
            match crossterm::event::read() {
                Ok(ev) => { if key_sender.send(ev).await.is_err() { break; } }
                Err(e) => eprintln!("Input error: {}", e),
            }
        }
    });

    write!(stdout(), "Arrow keys / space to move. ESC or 'q' to quit. 'd' to delete stored data.\r\n")
        .unwrap_or_default();

    loop {
        tokio::select!(
            Ok(sample) = subscriber.recv_async() => {
                match cdr::deserialize_from::<_, Log, _>(sample.payload().reader(), cdr::size::Infinite) {
                    Ok(log) => { println!("{}", log); stdout().execute(MoveToColumn(0)).unwrap(); }
                    Err(e) => eprintln!("Error decoding Log: {}", e),
                }
            },
            Some(event) = key_receiver.recv() => {
                match event {
                    Event::Key(KeyEvent { code: KeyCode::Up, .. }) =>
                        pub_twist(&publisher, 1.0 * linear_scale, 0.0).await,
                    Event::Key(KeyEvent { code: KeyCode::Down, .. }) =>
                        pub_twist(&publisher, -1.0 * linear_scale, 0.0).await,
                    Event::Key(KeyEvent { code: KeyCode::Left, .. }) =>
                        pub_twist(&publisher, 0.0, 1.0 * angular_scale).await,
                    Event::Key(KeyEvent { code: KeyCode::Right, .. }) =>
                        pub_twist(&publisher, 0.0, -1.0 * angular_scale).await,
                    Event::Key(KeyEvent { code: KeyCode::Char(' '), .. }) =>
                        pub_twist(&publisher, 0.0, 0.0).await,
                    Event::Key(KeyEvent { code: KeyCode::Char('d'), .. }) =>
                        del_twist(&publisher).await,
                    Event::Key(KeyEvent { code: KeyCode::Char('c'), modifiers, .. })
                        if modifiers.contains(KeyModifiers::CONTROL) => break,
                    Event::Key(KeyEvent { code: KeyCode::Esc, .. })
                    | Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => break,
                    _ => (),
                }
            }
        );
    }

    pub_twist(&publisher, 0.0, 0.0).await;
    crossterm::terminal::disable_raw_mode().unwrap();
}

fn build_config(args: &Args) -> Config {
    let mut config = match &args.config {
        Some(path) => Config::from_file(path).unwrap(),
        None => Config::default(),
    };
    if let Some(mode) = &args.mode {
        config.insert_json5("mode", &json!(mode).to_string()).unwrap();
    }
    if !args.connect.is_empty() {
        config.insert_json5("connect/endpoints", &json!(args.connect).to_string()).unwrap();
    }
    if !args.listen.is_empty() {
        config.insert_json5("listen/endpoints", &json!(args.listen).to_string()).unwrap();
    }
    if args.no_multicast_scouting {
        config.insert_json5("scouting/multicast/enabled", "false").unwrap();
    }
    config
}

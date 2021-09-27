//
// Copyright (c) 2021 ADLINK Technology Inc.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ADLINK zenoh team, <zenoh@adlink-labs.tech>
//
use async_std::channel::bounded;
use cdr::{CdrLe, Infinite};
use clap::{App, Arg};
use crossterm::{
    cursor::MoveToColumn,
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    ExecutableCommand,
};
use futures::prelude::*;
use futures::select;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use zenoh::net::*;
use zenoh::Properties;

#[derive(Serialize, PartialEq)]
struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Serialize, PartialEq)]
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

async fn pub_twist(session: &Session, cmd_key: &ResKey, linear: f64, angular: f64) {
    let twist = Twist {
        linear: Vector3 {
            x: linear,
            y: 0.0,
            z: 0.0,
        },
        angular: Vector3 {
            x: 0.0,
            y: 0.0,
            z: angular,
        },
    };

    let encoded = cdr::serialize::<_, _, CdrLe>(&twist, Infinite).unwrap();
    if let Err(e) = session.write(cmd_key, encoded.into()).await {
        log::warn!("Error writing to zenoh: {}", e);
    }
}

#[async_std::main]
async fn main() {
    // Initiate logging
    env_logger::init();

    let (config, cmd_vel, rosout, linear_scale, angular_scale) = parse_args();

    println!("Opening session...");
    let session = open(config.into()).await.unwrap();

    println!("Subscriber on {}", rosout);
    let sub_info = SubInfo {
        reliability: Reliability::Reliable,
        mode: SubMode::Push,
        period: None,
    };
    let mut subscriber = session
        .declare_subscriber(&rosout.into(), &sub_info)
        .await
        .unwrap();

    // ResKey for publication on "cmd_vel" topic
    let cmd_key = ResKey::from(cmd_vel);

    // Keyboard event read loop, sending each to an async_std channel
    // Note: enable raw mode for direct processing of key pressed, without having to hit ENTER...
    // Unfortunately, this mode doesn't process new line characters on output. Thus we have to call
    // `std::io::stdout().execute(MoveToColumn(0));` after each `println!`.
    crossterm::terminal::enable_raw_mode().unwrap();
    let (key_sender, key_receiver) = bounded::<Event>(10);
    async_std::task::spawn(async move {
        loop {
            match crossterm::event::read() {
                Ok(ev) => {
                    if let Err(e) = key_sender.send(ev).await {
                        log::warn!("Failed to push Key Event: {}", e);
                    }
                }
                Err(e) => {
                    log::warn!("Input error: {}", e);
                }
            }
        }
    });

    println!("Waiting commands with arrow keys or space bar to stop. Press on ESC, 'Q' or CTRL+C to quit.");
    std::io::stdout().execute(MoveToColumn(0)).unwrap();
    // Events management loop
    loop {
        select!(
            // On sample received by the subsriber
            sample = subscriber.stream().next().fuse() => {
                let sample = sample.unwrap();
                // copy to be removed if possible
                // let buf = sample.payload.to_vec();
                match cdr::deserialize_from::<_, Log, _>(sample.payload, cdr::size::Infinite) {
                    Ok(log) => {
                        println!("{}", log);
                        std::io::stdout().execute(MoveToColumn(0)).unwrap();
                    }
                    Err(e) => log::warn!("Error decoding Log: {}", e),
                }
            },

            // On keyboard event received from the async_std channel
            event = key_receiver.recv().fuse() => {
                match event {
                    Ok(Event::Key(KeyEvent { code: KeyCode::Up, modifiers: _ })) => {
                        pub_twist(&session, &cmd_key, 1.0 * linear_scale, 0.0).await
                    },
                    Ok(Event::Key(KeyEvent { code: KeyCode::Down, modifiers: _ })) => {
                        pub_twist(&session, &cmd_key, -1.0 * linear_scale, 0.0).await
                    },
                    Ok(Event::Key(KeyEvent { code: KeyCode::Left, modifiers: _ })) => {
                        pub_twist(&session, &cmd_key, 0.0, 1.0 * angular_scale).await
                    },
                    Ok(Event::Key(KeyEvent { code: KeyCode::Right, modifiers: _ })) => {
                        pub_twist(&session, &cmd_key, 0.0, -1.0 * angular_scale).await
                    },
                    Ok(Event::Key(KeyEvent { code: KeyCode::Char(' '), modifiers: _ })) => {
                        pub_twist(&session, &cmd_key, 0.0, 0.0).await
                    },
                    Ok(Event::Key(KeyEvent { code: KeyCode::Esc, modifiers: _ })) |
                    Ok(Event::Key(KeyEvent { code: KeyCode::Char('q'), modifiers: _ })) => {
                        break
                    },
                    Ok(Event::Key(KeyEvent { code: KeyCode::Char('c'), modifiers })) => {
                        if modifiers.contains(KeyModifiers::CONTROL) { break }
                    },
                    Ok(_) => (),
                    Err(e) => {
                        log::warn!("Input error: {}", e);
                    }
                }
            }
        );
    }

    // Stop robot at exit
    pub_twist(&session, &cmd_key, 0.0, 0.0).await;

    crossterm::terminal::disable_raw_mode().unwrap();
}

fn parse_args() -> (Properties, String, String, f64, f64) {
    let args = App::new("zenoh-net sub example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE]  'The zenoh session mode (peer by default).")
                .possible_values(&["peer", "client"]),
        )
        .arg(Arg::from_usage(
            "-e, --peer=[LOCATOR]...   'Peer locators used to initiate the zenoh session.'",
        ))
        .arg(Arg::from_usage(
            "-l, --listener=[LOCATOR]...   'Locators to listen on.'",
        ))
        .arg(Arg::from_usage(
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .arg(
            Arg::from_usage("--cmd_vel=[topic] 'The 'cmd_vel' ROS2 topic'")
                .default_value("/rt/turtle1/cmd_vel"),
        )
        .arg(
            Arg::from_usage("--rosout=[topic] 'The 'rosout' ROS2 topic'")
                .default_value("/rt/rosout"),
        )
        .arg(
            Arg::from_usage("-a, --angular_scale=[FLOAT] 'The angular scale.'")
                .default_value("2.0"),
        )
        .arg(Arg::from_usage("-x, --linear_scale=[FLOAT] 'The linear scale.").default_value("2.0"))
        .get_matches();

    let mut config = if let Some(conf_file) = args.value_of("config") {
        Properties::from(std::fs::read_to_string(conf_file).unwrap())
    } else {
        Properties::default()
    };
    for key in ["mode", "peer", "listener"].iter() {
        if let Some(value) = args.values_of(key) {
            config.insert(key.to_string(), value.collect::<Vec<&str>>().join(","));
        }
    }
    if args.is_present("no-multicast-scouting") {
        config.insert("multicast_scouting".to_string(), "false".to_string());
    }

    let cmd_vel = args.value_of("cmd_vel").unwrap().to_string();
    let rosout = args.value_of("rosout").unwrap().to_string();
    let angular_scale: f64 = args.value_of("angular_scale").unwrap().parse().unwrap();
    let linear_scale: f64 = args.value_of("linear_scale").unwrap().parse().unwrap();

    (config, cmd_vel, rosout, angular_scale, linear_scale)
}

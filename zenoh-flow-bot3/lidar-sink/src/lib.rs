//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

use async_trait::async_trait;
use zenoh::config::Config;
use zenoh::prelude::*;
use zenoh::Session;
use zenoh_flow::async_std::sync::{Arc, Mutex};
use zenoh_flow::runtime::message::DataMessage;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::Configuration;
use zenoh_flow::{export_sink, types::ZFResult, Node, State};
use zenoh_flow::{Context, Sink};
use zenoh_flow_example_types::ros2::tb3::LaserScan;

use std::fs::File;
use std::io::Write;

struct LaserSink;

#[derive(ZFState, Clone, Debug)]
struct SinkState {
    pub file: Option<Arc<Mutex<File>>>,
    pub zsession: Option<Arc<Session>>,
}

impl SinkState {
    pub fn new(configuration: &Option<Configuration>) -> Self {
        let (file, zsession) = match configuration {
            Some(c) => {
                let file = match c["file"].as_str() {
                    Some(f) => Some(Arc::new(Mutex::new(File::create(f).unwrap()))),
                    None => None,
                };

                let zsession = match c["locator"].as_str() {
                    Some(l) => {
                        let mut zconfig = Config::default();
                        zconfig
                            .set_mode(Some(zenoh::config::WhatAmI::Client))
                            .unwrap();
                        zconfig.connect.endpoints.push(l.parse().unwrap());

                        Some(Arc::new(zenoh::open(zconfig).wait().unwrap()))
                    }
                    None => None,
                };

                (file, zsession)
            }
            None => (None, None),
        };

        Self { file, zsession }
    }
}

#[async_trait]
impl Sink for LaserSink {
    async fn run(
        &self,
        _context: &mut Context,
        dyn_state: &mut State,
        mut input: DataMessage,
    ) -> ZFResult<()> {
        let state = dyn_state.try_get::<SinkState>()?;

        let data = input.get_inner_data().try_get::<LaserScan>()?;

        match &state.file {
            None => {
                println!("#######");
                println!("Laser Sink Received -> {:?}", data);
                println!("#######");
            }
            Some(f) => {
                let mut guard = f.lock().await;
                writeln!(&mut guard, "#######").unwrap();
                writeln!(&mut guard, "Laser Sink Received -> {:?}", data).unwrap();
                writeln!(&mut guard, "#######").unwrap();
                guard.sync_all().unwrap();
            }
        }
        if let Some(zsession) = &state.zsession {
            let serialized = serde_json::to_string(&data.0).unwrap();
            zsession.put("/zf-bot/lidar", serialized).await.unwrap();
        }
        Ok(())
    }
}

impl Node for LaserSink {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        Ok(State::from(SinkState::new(configuration)))
    }

    fn finalize(&self, dyn_state: &mut State) -> ZFResult<()> {
        let state = dyn_state.try_get::<SinkState>()?;

        match &mut state.file {
            None => Ok(()),
            Some(_) => {
                state.file = None;
                Ok(())
            }
        }
    }
}

export_sink!(register);

fn register() -> ZFResult<Arc<dyn Sink>> {
    Ok(Arc::new(LaserSink) as Arc<dyn Sink>)
}

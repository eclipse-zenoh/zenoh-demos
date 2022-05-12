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
use zenoh_flow::async_std::sync::{Arc, Mutex};
use zenoh_flow::runtime::message::DataMessage;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::Configuration;
use zenoh_flow::{export_sink, types::ZFResult, Node, State};
use zenoh_flow::{Context, Sink};
use zenoh_flow_example_types::ros2::tb3::RobotInformation;

use std::fs::File;
use std::io::Write;

struct RobotSink;

#[derive(ZFState, Clone, Debug)]
struct SinkState {
    pub file: Option<Arc<Mutex<File>>>,
}

impl SinkState {
    pub fn new(configuration: &Option<Configuration>) -> Self {
        let file = match configuration {
            Some(c) => {
                let f = File::create(c["file"].as_str().unwrap()).unwrap();
                Some(Arc::new(Mutex::new(f)))
            }
            None => None,
        };
        Self { file }
    }
}

#[async_trait]
impl Sink for RobotSink {
    async fn run(
        &self,
        _context: &mut Context,
        dyn_state: &mut State,
        mut input: DataMessage,
    ) -> ZFResult<()> {
        let state = dyn_state.try_get::<SinkState>()?;

        let data = input.get_inner_data().try_get::<RobotInformation>()?;

        match &state.file {
            None => {
                println!("#######");
                println!("Robot Sink Received -> {:?}", data);
                println!("#######");
            }
            Some(f) => {
                let mut guard = f.lock().await;
                writeln!(&mut guard, "#######").unwrap();
                writeln!(&mut guard, "Robot Sink Received -> {:?}", data).unwrap();
                writeln!(&mut guard, "#######").unwrap();
                guard.sync_all().unwrap();
            }
        }
        Ok(())
    }
}

impl Node for RobotSink {
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
    Ok(Arc::new(RobotSink) as Arc<dyn Sink>)
}

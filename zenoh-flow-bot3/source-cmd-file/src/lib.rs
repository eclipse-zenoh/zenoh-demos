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

use async_std::sync::Arc;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use zenoh_flow::{
    zenoh_flow_derive::ZFState, Configuration, Context, Data, Node, Source, State, ZFResult,
};
use zenoh_flow_example_types::GamepadInput;

#[derive(Debug, ZFState)]
pub struct GamepadState {
    reader: BufReader<File>,
    input: GamepadInput,
}

pub struct GamepadSource;

impl Node for GamepadSource {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {

        let file = match configuration {
            Some(c) => {
                File::open(c["file"].as_str().unwrap()).unwrap()
            },
            None => File::open("/tmp/commands-flow.txt")?,
        };

        Ok(State::from(GamepadState {
            reader: BufReader::new(file),
            input: GamepadInput::default(),
        }))
    }

    fn finalize(&self, _: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

#[async_trait]
impl Source for GamepadSource {
    async fn run(&self, _: &mut Context, state: &mut State) -> ZFResult<Data> {
        let mut state = state.try_get::<GamepadState>()?;

        // Read line and execute action
        let mut line = String::new();
        state.reader.read_line(&mut line)?;
        match line.as_str() {
            "left\n" => {
                state.input.left_trigger = 0.0;
                state.input.right_trigger = 0.0;
                state.input.left_stick_x = -1.0;
            }
            "right\n" => {
                state.input.left_trigger = 0.0;
                state.input.right_trigger = 0.0;
                state.input.left_stick_x = 1.0;
            }
            "forward\n" => {
                state.input.left_trigger = 0.0;
                state.input.right_trigger = 1.0;
                state.input.left_stick_x = 0.0;
            }
            "backward\n" => {
                state.input.left_trigger = 1.0;
                state.input.right_trigger = 0.0;
                state.input.left_stick_x = 0.0;
            }
            "stop\n" => {
                state.input.left_trigger = 0.0;
                state.input.right_trigger = 0.0;
                state.input.left_stick_x = 0.0;
            }
            line => println!("I read {line} so I do know what to do..."),
        }

        Ok(Data::from(state.input))
    }
}

zenoh_flow::export_source!(register);

fn register() -> ZFResult<Arc<dyn Source>> {
    Ok(Arc::new(GamepadSource) as Arc<dyn Source>)
}

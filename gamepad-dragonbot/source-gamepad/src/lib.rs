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
use gilrs::{EventType, Gilrs};
use std::sync::{Arc, Mutex};
use types::GamepadInput;
use zenoh_flow::{
    zenoh_flow_derive::ZFState, Configuration, Context, Data, Node, Source, State, ZFError,
    ZFResult,
};

#[derive(Debug, ZFState)]
pub struct GamepadState {
    gilrs: Arc<Mutex<Gilrs>>,
    input: GamepadInput,
}

pub struct GamepadSource;

impl Node for GamepadSource {
    fn initialize(&self, _: &Option<Configuration>) -> ZFResult<State> {
        let gilrs = Gilrs::new().expect("Could not start Gilrs");
        Ok(State::from(GamepadState {
            gilrs: Arc::new(Mutex::new(gilrs)),
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
        let state = state.try_get::<GamepadState>()?;
        let mut gilrs = state.gilrs.lock().map_err(|_| ZFError::GenericError)?;

        while let Some(event) = gilrs.next_event() {
            match event.event {
                EventType::ButtonChanged(button, value, _) => match button {
                    gilrs::Button::LeftTrigger2 => state.input.left_trigger = value,
                    gilrs::Button::RightTrigger2 => state.input.right_trigger = value,
                    _ => (),
                },
                EventType::AxisChanged(stick, value, _) => {
                    if stick == gilrs::Axis::LeftStickX {
                        state.input.left_stick_x = value
                    }
                }
                // Ignore all other events
                _ => (),
            }
        }

        Ok(Data::from(state.input))
    }
}

zenoh_flow::export_source!(register);

fn register() -> ZFResult<Arc<dyn Source>> {
    Ok(Arc::new(GamepadSource) as Arc<dyn Source>)
}

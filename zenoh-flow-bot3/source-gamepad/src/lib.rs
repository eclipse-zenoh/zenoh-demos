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

use async_std::sync::{Arc, Mutex};
use async_trait::async_trait;
use gilrs::{EventType, Gilrs};
use std::time::Duration;
use zenoh_flow::prelude::*;
use zenoh_flow_example_types::GamepadInput;

#[derive(Debug)]
pub struct GamepadState {
    pub(crate) gilrs: Gilrs,
    pub(crate) input: GamepadInput,
}

impl GamepadState {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            gilrs: Gilrs::new().expect("Could not start Gilrs"),
            input: GamepadInput::default(),
        }))
    }
}

pub struct GamepadSource;

#[async_trait]
impl Source for GamepadSource {
    async fn setup(
        &self,
        _context: &mut Context,
        _configuration: &Option<Configuration>,
        mut outputs: Outputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
        let output = outputs.take_into_arc("gamepad-input").unwrap();
        let gamepad_state = GamepadState::new();

        // TODO(Julien L.) Source is periodic: 100ms.
        Ok(Some(Box::new(move || {
            let output = Arc::clone(&output);
            let gamepad_state = Arc::clone(&gamepad_state);

            async move {
                let data: Data;
                // CAVEAT: An explicit scope is needed to tell Rust that the MutexGuard is dropped
                // before the next `await` is called.
                {
                    let mut gamepad = gamepad_state.lock().await;
                    while let Some(event) = gamepad.gilrs.next_event() {
                        match event.event {
                            EventType::ButtonChanged(button, value, _) => match button {
                                gilrs::Button::LeftTrigger2 => gamepad.input.left_trigger = value,
                                gilrs::Button::RightTrigger2 => gamepad.input.right_trigger = value,
                                _ => (),
                            },
                            EventType::AxisChanged(stick, value, _) => {
                                if stick == gilrs::Axis::LeftStickX {
                                    gamepad.input.left_stick_x = value
                                }
                            }
                            // Ignore all other events
                            _ => (),
                        }
                    }
                    data = Data::from(gamepad.input);
                }

                output.send_async(data, None).await?;
                async_std::task::sleep(Duration::from_millis(100)).await;

                Ok(())
            }
        })))
    }
}

zenoh_flow::export_source!(register);

fn register() -> Result<Arc<dyn Source>> {
    Ok(Arc::new(GamepadSource) as Arc<dyn Source>)
}

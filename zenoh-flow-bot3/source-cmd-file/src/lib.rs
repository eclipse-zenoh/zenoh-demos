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

use async_std::sync::Arc;
use async_std::sync::Mutex;
use async_trait::async_trait;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use zenoh_flow::prelude::*;
use zenoh_flow_example_types::GamepadInput;

pub struct GamepadSource;

#[async_trait]
impl Source for GamepadSource {
    async fn setup(
        &self,
        _context: &mut Context,
        configuration: &Option<Configuration>,
        mut outputs: Outputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
        let file = match configuration {
            Some(c) => File::open(c["file"].as_str().unwrap()).unwrap(),
            None => File::open("/tmp/commands-flow.txt")?,
        };
        let reader = Arc::new(Mutex::new(BufReader::new(file)));
        let output = outputs
            .take_into_arc("out")
            .expect("Could not find output < out >");

        Ok(Some(Box::new(move || {
            let output = Arc::clone(&output);
            let reader = Arc::clone(&reader);

            async move {
                let mut gamepad_input = GamepadInput::default();
                let mut line = String::new();
                {
                    let mut reader = reader.lock().await;
                    reader.read_line(&mut line)?;
                }

                match line.as_str() {
                    "left\n" => {
                        gamepad_input.left_trigger = 0.0;
                        gamepad_input.right_trigger = 0.0;
                        gamepad_input.left_stick_x = -1.0;
                    }
                    "right\n" => {
                        gamepad_input.left_trigger = 0.0;
                        gamepad_input.right_trigger = 0.0;
                        gamepad_input.left_stick_x = 1.0;
                    }
                    "forward\n" => {
                        gamepad_input.left_trigger = 0.0;
                        gamepad_input.right_trigger = 1.0;
                        gamepad_input.left_stick_x = 0.0;
                    }
                    "backward\n" => {
                        gamepad_input.left_trigger = 1.0;
                        gamepad_input.right_trigger = 0.0;
                        gamepad_input.left_stick_x = 0.0;
                    }
                    "stop\n" => {
                        gamepad_input.left_trigger = 0.0;
                        gamepad_input.right_trigger = 0.0;
                        gamepad_input.left_stick_x = 0.0;
                    }
                    line => println!("I read {line} so I do know what to do..."),
                }

                output.send_async(Data::from(gamepad_input), None).await?;

                Ok(())
            }
        })))
    }
}

zenoh_flow::export_source!(register);

fn register() -> Result<Arc<dyn Source>> {
    Ok(Arc::new(GamepadSource) as Arc<dyn Source>)
}

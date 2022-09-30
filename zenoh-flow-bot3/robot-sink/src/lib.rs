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
use std::fs::File;
use std::io::Write;
use zenoh_flow::prelude::*;
use zenoh_flow_example_types::ros2::tb3::RobotInformation;

struct RobotSink;

#[async_trait]
impl Sink for RobotSink {
    async fn setup(
        &self,
        context: &mut Context,
        configuration: &Option<Configuration>,
        mut inputs: Inputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
        let file = match configuration {
            Some(c) => {
                let f = File::create(c["file"].as_str().unwrap()).unwrap();
                Some(Arc::new(Mutex::new(f)))
            }
            None => None,
        };

        let input = inputs.take("Data").expect("No input called 'Data'");

        context.register_input_callback(
            input,
            Box::new(move |message| {
                let file = file.clone();

                async move {
                    if let Message::Data(mut robot_information_raw) = message {
                        let robot_information = robot_information_raw
                            .get_inner_data()
                            .try_get::<RobotInformation>()?;

                        match file {
                            Some(file) => {
                                let mut guard = file.lock().await;
                                writeln!(&mut guard, "#######").unwrap();
                                writeln!(
                                    &mut guard,
                                    "Robot Sink Received -> {:?}",
                                    robot_information
                                )
                                .unwrap();
                                writeln!(&mut guard, "#######").unwrap();
                                guard.sync_all().unwrap();
                            }
                            None => {
                                println!("#######");
                                println!("Robot Sink Received -> {:?}", robot_information);
                                println!("#######");
                            }
                        }
                    }

                    Ok(())
                }
            }),
        );

        Ok(None)
    }
}

export_sink!(register);

fn register() -> Result<Arc<dyn Sink>> {
    Ok(Arc::new(RobotSink) as Arc<dyn Sink>)
}

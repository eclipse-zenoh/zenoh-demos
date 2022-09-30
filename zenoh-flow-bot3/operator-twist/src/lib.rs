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

use std::sync::Arc;
use zenoh_flow::{bail, prelude::*};
use zenoh_flow_example_types::{ros2::geometry::Twist, GamepadInput};

const INPUT_PORT_ID: &str = "gamepad-input";
const OUTPUT_PORT_ID: &str = "twist";

pub struct OperatorTwist;

#[async_trait::async_trait]
impl Operator for OperatorTwist {
    async fn setup(
        &self,
        _ctx: &mut Context,
        _configuration: &Option<Configuration>,
        mut inputs: Inputs,
        mut outputs: Outputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
        let input = inputs.take_into_arc(INPUT_PORT_ID).unwrap();
        let output = outputs.take_into_arc(OUTPUT_PORT_ID).unwrap();

        Ok(Some(Box::new(move || {
            let input = Arc::clone(&input);
            let output = Arc::clone(&output);

            async move {
                let input_raw = input.recv_async().await?;
                if let Message::Data(mut gamepad_input_raw) = input_raw {
                    let gamepad_input = gamepad_input_raw
                        .get_inner_data()
                        .try_get::<GamepadInput>()?;
                    let twist: Twist = gamepad_input.into();
                    output.send_async(Data::from(twist), None).await
                } else {
                    bail!(ErrorKind::InvalidData, "Did not receive a Message::Data")
                }
            }
        })))
    }
}

zenoh_flow::export_operator!(register);

fn register() -> zenoh_flow::Result<Arc<dyn Operator>> {
    Ok(Arc::new(OperatorTwist) as Arc<dyn Operator>)
}

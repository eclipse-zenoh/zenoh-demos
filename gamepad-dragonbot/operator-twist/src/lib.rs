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

use std::{collections::HashMap, sync::Arc};
use types::{GamepadInput, Twist};
use zenoh_flow::{
    default_input_rule, default_output_rule, zf_empty_state, Data, Node, Operator, PortId, ZFError,
};

const INPUT_PORT_ID: &str = "gamepad-input";
const OUTPUT_PORT_ID: &str = "twist";

pub struct OperatorTwist;

impl Node for OperatorTwist {
    fn initialize(
        &self,
        _: &Option<zenoh_flow::Configuration>,
    ) -> zenoh_flow::ZFResult<zenoh_flow::State> {
        zf_empty_state!()
    }

    fn finalize(&self, _: &mut zenoh_flow::State) -> zenoh_flow::ZFResult<()> {
        Ok(())
    }
}

impl Operator for OperatorTwist {
    fn input_rule(
        &self,
        _: &mut zenoh_flow::Context,
        state: &mut zenoh_flow::State,
        tokens: &mut std::collections::HashMap<zenoh_flow::PortId, zenoh_flow::InputToken>,
    ) -> zenoh_flow::ZFResult<bool> {
        default_input_rule(state, tokens)
    }

    fn run(
        &self,
        _: &mut zenoh_flow::Context,
        _: &mut zenoh_flow::State,
        inputs: &mut std::collections::HashMap<zenoh_flow::PortId, zenoh_flow::DataMessage>,
    ) -> zenoh_flow::ZFResult<std::collections::HashMap<zenoh_flow::PortId, zenoh_flow::Data>> {
        let mut outputs = HashMap::<PortId, Data>::with_capacity(1);
        let mut gamepad_input_raw = inputs
            .remove(INPUT_PORT_ID)
            .ok_or_else(|| ZFError::InvalidData("No data".to_string()))?;

        let gamepad_input = gamepad_input_raw
            .get_inner_data()
            .try_get::<GamepadInput>()?;

        outputs.insert(
            OUTPUT_PORT_ID.into(),
            Data::from::<Twist>(gamepad_input.into()),
        );

        Ok(outputs)
    }

    fn output_rule(
        &self,
        _: &mut zenoh_flow::Context,
        state: &mut zenoh_flow::State,
        outputs: std::collections::HashMap<zenoh_flow::PortId, zenoh_flow::Data>,
        _: Option<zenoh_flow::LocalDeadlineMiss>,
    ) -> zenoh_flow::ZFResult<std::collections::HashMap<zenoh_flow::PortId, zenoh_flow::NodeOutput>>
    {
        default_output_rule(state, outputs)
    }
}

zenoh_flow::export_operator!(register);

fn register() -> zenoh_flow::ZFResult<Arc<dyn Operator>> {
    Ok(Arc::new(OperatorTwist) as Arc<dyn Operator>)
}

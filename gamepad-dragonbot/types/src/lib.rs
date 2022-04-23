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

use serde::{Deserialize, Serialize};
use zenoh_flow::zenoh_flow_derive::ZFData;
use zenoh_flow::{Deserializable, ZFData, ZFError, ZFResult};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, ZFData)]
pub struct GamepadInput {
    pub left_trigger: f32,
    pub right_trigger: f32,
    pub left_stick_x: f32,
}

impl Default for GamepadInput {
    fn default() -> Self {
        Self {
            left_trigger: 0.0,
            right_trigger: 0.0,
            left_stick_x: 0.0,
        }
    }
}

impl ZFData for GamepadInput {
    fn try_serialize(&self) -> ZFResult<Vec<u8>> {
        bincode::serialize(self).map_err(|_| ZFError::SerializationError)
    }
}

impl Deserializable for GamepadInput {
    fn try_deserialize(bytes: &[u8]) -> ZFResult<Self>
    where
        Self: Sized,
    {
        bincode::deserialize(bytes).map_err(|_| ZFError::DeseralizationError)
    }
}

const LINEAR_SCALING_FACTOR: f32 = 0.20;
const ANGULAR_SCALING_FACTOR: f32 = 2.60;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, ZFData)]
pub struct Twist {
    pub linear: f32,
    pub angular: f32,
}

impl ZFData for Twist {
    fn try_serialize(&self) -> ZFResult<Vec<u8>> {
        bincode::serialize(self).map_err(|_| ZFError::SerializationError)
    }
}

impl Deserializable for Twist {
    fn try_deserialize(bytes: &[u8]) -> ZFResult<Self>
    where
        Self: Sized,
    {
        bincode::deserialize(bytes).map_err(|_| ZFError::DeseralizationError)
    }
}

impl From<&GamepadInput> for Twist {
    fn from(gamepad_input: &GamepadInput) -> Self {
        // left trigger indicates going backward
        // right trigger indicates going forward
        let linear =
            (gamepad_input.right_trigger - gamepad_input.left_trigger) * LINEAR_SCALING_FACTOR;

        // left stick x indicates going left / right.
        // However, it feels more natural if the values are swapped, hence the minus in front.
        let angular = -gamepad_input.left_stick_x * ANGULAR_SCALING_FACTOR;

        Self { linear, angular }
    }
}

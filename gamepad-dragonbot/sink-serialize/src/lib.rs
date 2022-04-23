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
use std::sync::Arc;
use types::Twist;
use zenoh::prelude::ZFuture;
use zenoh_flow::{zenoh_flow_derive::ZFState, Context, DataMessage, Node, Sink, State, ZFResult};

#[derive(Debug, ZFState)]
pub struct SinkState {
    buffer: [u8; 48],
    session: Option<zenoh::Session>,
    expr_id: u64,
}

pub struct SinkSerialize;

impl Node for SinkSerialize {
    fn initialize(
        &self,
        _: &Option<zenoh_flow::Configuration>,
    ) -> zenoh_flow::ZFResult<zenoh_flow::State> {
        // FIXME
        let mut config = zenoh::config::default();
        config
            .connect
            .set_endpoints(
                vec!["tcp/192.168.86.12:7447"]
                    .iter()
                    .filter_map(|l| l.parse().ok())
                    .collect(),
            )
            .expect("Could not set locator");

        let session = zenoh::open(config).wait().expect("Could not open Session.");
        let expr_id = session
            .declare_expr("/rt/cmd_vel")
            .wait()
            .expect("Could not declare expression.");
        Ok(State::from(SinkState {
            buffer: [0u8; 48],
            session: Some(session),
            expr_id,
        }))
    }

    fn finalize(&self, dyn_state: &mut zenoh_flow::State) -> zenoh_flow::ZFResult<()> {
        let state = dyn_state.try_get::<SinkState>()?;
        if let Some(session) = &state.session {
            session
                .undeclare_expr(state.expr_id)
                .wait()
                .expect("Could not undeclare expr");
        }

        state
            .session
            .take()
            .unwrap()
            .close()
            .wait()
            .expect("Could not close Session");
        Ok(())
    }
}

#[async_trait]
impl Sink for SinkSerialize {
    async fn run(
        &self,
        _: &mut Context,
        dyn_state: &mut State,
        mut input: DataMessage,
    ) -> ZFResult<()> {
        let twist = input.get_inner_data().try_get::<Twist>()?;
        let state = dyn_state.try_get::<SinkState>()?;

        // The way the robot works, 3 floats are expected for the linear "velocity". However, only
        // the first one matters. Hence, we serialize it on 0..8.
        serialize_avr_float_64(&mut state.buffer[0..8], twist.linear);
        // Similarly, 3 floats are expected for the angular "velocity" and only the last one
        // matters. So we serialize it on 40..48.
        serialize_avr_float_64(&mut state.buffer[40..48], twist.angular);

        if let Some(session) = &state.session {
            session
                .put(state.expr_id, &state.buffer[..])
                .wait()
                .expect("Could not put data.");
        }

        Ok(())
    }
}

// See: https://github.com/gabrik/zenoh-demos/blob/master/zenoh-dragonbot/zenoh-pico-teleop-sub/include/turtlebot3_ros_lib/ros/msg.h#L64
fn serialize_avr_float_64(buffer: &mut [u8], value: f32) {
    if buffer.len() < 8 {
        // FIXME
        panic!("I need more memory.");
    }

    let val = value.to_bits() as i32;
    let mut exp = (val >> 23) & 255;
    if exp != 0 {
        exp += 1023 - 127;
    }

    buffer[0] = 0;
    buffer[1] = 0;
    buffer[2] = 0;
    buffer[3] = (val << 5) as u8;
    buffer[4] = (val >> 3) as u8;
    buffer[5] = (val >> 11) as u8;
    buffer[6] = (((exp << 4) as u8) & 0xf0) | (((val >> 19) as u8) & 0x0f);
    buffer[7] = ((exp >> 4) as u8) & 0x7F;

    if value.is_sign_negative() {
        buffer[7] |= 0x80;
    }
}

zenoh_flow::export_sink!(register);

fn register() -> ZFResult<Arc<dyn Sink>> {
    Ok(Arc::new(SinkSerialize) as Arc<dyn Sink>)
}

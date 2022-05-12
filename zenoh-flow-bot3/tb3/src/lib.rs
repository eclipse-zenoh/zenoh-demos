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
use std::collections::HashMap;
use std::time::Duration;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{
    default_output_rule, zf_spin_lock, Configuration, Data, LocalDeadlineMiss, Node, NodeOutput,
    Operator, PortId, State, ZFError, ZFResult,
};
use zenoh_flow_example_types::ros2::geometry::{Quaternion, Twist, Vector3};
use zenoh_flow_example_types::ros2::sensors::{
    BatteryState, JointState, MagneticField, PowerSupplyHealth, PowerSupplyStatus,
    PowerSupplyTechnology, IMU,
};
use zenoh_flow_example_types::ros2::tb3::{RobotInformation, SensorState};

mod addresses;

const INPUT_TICK_ID: &str = "Tick";
const INPUT_TWIST_ID: &str = "Twist";
const OUTPUT_PORT_ID: &str = "Robot";

// ref) http://emanual.robotis.com/docs/en/dxl/x/xl430-w250/#goal-velocity104
const RPM_TO_MS: f64 = 0.229 * 0.0034557519189487725;

// 0.087890625[deg] * 3.14159265359 / 180 = 0.001533981f
const TICK_TO_RAD: f64 = 0.001533981;

#[derive(Debug)]
struct Tb3;

#[derive(ZFState, Clone)]
struct TB3State {
    pub serial: String,
    pub delay: f64,
    pub bus: Arc<Mutex<dynamixel2::Bus<Vec<u8>, Vec<u8>>>>,
    pub count: u8,
}

// because of dynamixel::Bus
impl std::fmt::Debug for TB3State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "TB3State: serial:{:?} delay:{:?}",
            self.serial, self.delay
        )
    }
}

impl TryFrom<&Option<Configuration>> for TB3State {
    type Error = ZFError;

    fn try_from(configuration: &Option<Configuration>) -> Result<Self, Self::Error> {
        async_std::task::block_on(async {
            let (serial, delay, baudrate) = match configuration {
                Some(configuration) => {
                    let serial = match configuration["serial"].as_str() {
                        Some(configured_serial) => configured_serial.to_string(),
                        None => "/dev/ttyACM0".to_string(),
                    };

                    let delay = configuration["delay"].as_f64().unwrap_or(1000.0);

                    let baudrate = configuration["baudrate"].as_u64().unwrap_or(1000000) as u32;

                    (serial, delay, baudrate)
                }

                None => ("/dev/ttyACM0".to_string(), 1000.0, 1000000),
            };

            let bus = match dynamixel2::Bus::open(serial.clone(), baudrate, Duration::from_secs(3))
            {
                Ok(mut bus) => {
                    bus.write_u8(200, addresses::IMU_RE_CALIBRATION, 1)
                        .await
                        .map_err(|e| {
                            ZFError::InvalidData(format!("TB3 Init write error: {}", e))
                        })?;

                    async_std::task::sleep(Duration::from_secs(5)).await;

                    let status = bus
                        .read_u8(200, addresses::DEVICE_STATUS)
                        .await
                        .map_err(|e| ZFError::InvalidData(format!("TB3 Init read error: {}", e)))?;
                    if status == 255 {
                        return Err(ZFError::InvalidData("Motor not connected!".to_string()));
                    }

                    Ok(bus)
                }
                Err(e) => Err(ZFError::InvalidData(format!("TB3 Init error: {}", e))),
            }?;

            Ok(Self {
                serial,
                delay,
                bus: Arc::new(Mutex::new(bus)),
                count: 0u8,
            })
        })
    }
}

impl Operator for Tb3 {
    fn input_rule(
        &self,
        _context: &mut zenoh_flow::Context,
        _state: &mut State,
        _tokens: &mut HashMap<PortId, zenoh_flow::InputToken>,
    ) -> ZFResult<bool> {
        Ok(true)
    }

    fn run(
        &self,
        _context: &mut zenoh_flow::Context,
        dyn_state: &mut State,
        inputs: &mut HashMap<PortId, zenoh_flow::runtime::message::DataMessage>,
    ) -> ZFResult<HashMap<PortId, Data>> {
        async_std::task::block_on(async {
            let mut results = HashMap::<PortId, Data>::with_capacity(1);

            let mut state = dyn_state.try_get::<TB3State>()?;
            let mut bus = zf_spin_lock!(state.bus);

            state.count = state.count.wrapping_add(1);
            bus.write_u8(200, addresses::HEARTBEAT, state.count)
                .await
                .map_err(|e| ZFError::InvalidData(format!("TB3 Heartbeat error: {}", e)))?;

            // If tick received to readings
            match inputs.remove(INPUT_TICK_ID) {
                Some(_tick) => {
                    let battery = self.get_battery_state(&mut bus).await?;
                    let imu = self.get_imu(&mut bus).await?;
                    let magnetic_field = self.get_magnetic(&mut bus).await?;
                    let joint_state = self.get_joint_states(&mut bus).await?;
                    let sensor_state = self.get_sensor_state(&mut bus).await?;

                    let robot_information = RobotInformation {
                        battery,
                        imu,
                        magnetic_field,
                        joint_state,
                        sensor_state,
                    };

                    results.insert(
                        OUTPUT_PORT_ID.into(),
                        Data::from::<RobotInformation>(robot_information),
                    );
                }
                None => (),
            };

            // If twist received move the robot
            match inputs.remove(INPUT_TWIST_ID) {
                Some(mut twist) => {
                    let data = twist.get_inner_data().try_get::<Twist>()?;
                    println!("TB3 RECEIVED TWIST {:?}", data);
                    self.write_to_motors(&mut bus, data).await?;
                }
                None => (),
            }

            Ok(results)
        })
    }

    fn output_rule(
        &self,
        _context: &mut zenoh_flow::Context,
        state: &mut State,
        outputs: HashMap<PortId, Data>,
        _deadlinemiss: Option<LocalDeadlineMiss>,
    ) -> ZFResult<HashMap<PortId, NodeOutput>> {
        default_output_rule(state, outputs)
    }
}

impl Node for Tb3 {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        Ok(State::from(TB3State::try_from(configuration)?))
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

impl Tb3 {
    async fn get_battery_state(
        &self,
        bus: &mut dynamixel2::Bus<Vec<u8>, Vec<u8>>,
    ) -> ZFResult<BatteryState> {
        let design_capacity = 4.0f32;

        let voltage = f32::from_bits(
            bus.read_u32(200, addresses::BATTERY_VOLTAGE)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!("TB3 Unable to read battery voltage error: {}", e))
                })?,
        ) * 0.01f32;
        let percentage = f32::from_bits(
            bus.read_u32(200, addresses::BATTERY_PERCENTAGE)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!(
                        "TB3 Unable to read battery percentage error: {}",
                        e
                    ))
                })?,
        ) * 0.01f32;

        let present = voltage > 7.0f32;

        Ok(BatteryState {
            voltage,
            temperature: 0f32,
            current: 0f32,
            charge: 0f32,
            capacity: 0f32,
            design_capacity,
            percentage,
            power_supply_status: PowerSupplyStatus::Unknown,
            power_supply_health: PowerSupplyHealth::Unknown,
            power_supply_technology: PowerSupplyTechnology::LiPO,
            present,
            cell_voltage: vec![],
            cell_temperature: vec![],
            location: "Robot".to_string(),
            serial_number: "TOR-4000LI3S30D".to_string(),
        })
    }

    async fn get_imu(&self, bus: &mut dynamixel2::Bus<Vec<u8>, Vec<u8>>) -> ZFResult<IMU> {
        let orientation_w = f32::from_bits(
            bus.read_u32(200, addresses::IMU_ORIENTATION_W)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!("TB3 Unable to read imu orientation error: {}", e))
                })?,
        );

        let orientation_x = f32::from_bits(
            bus.read_u32(200, addresses::IMU_ORIENTATION_X)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!("TB3 Unable to read imu orientation error: {}", e))
                })?,
        );

        let orientation_y = f32::from_bits(
            bus.read_u32(200, addresses::IMU_ORIENTATION_Y)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!("TB3 Unable to read imu orientation error: {}", e))
                })?,
        );

        let orientation_z = f32::from_bits(
            bus.read_u32(200, addresses::IMU_ORIENTATION_Z)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!("TB3 Unable to read imu orientation error: {}", e))
                })?,
        );

        let velocity_x = f32::from_bits(
            bus.read_u32(200, addresses::IMU_ANGULAR_VELOCITY_X)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!("TB3 Unable to read imu velocity error: {}", e))
                })?,
        );

        let velocity_y = f32::from_bits(
            bus.read_u32(200, addresses::IMU_ANGULAR_VELOCITY_Y)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!("TB3 Unable to read imu velocity error: {}", e))
                })?,
        );

        let velocity_z = f32::from_bits(
            bus.read_u32(200, addresses::IMU_ANGULAR_VELOCITY_Z)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!("TB3 Unable to read imu velocity error: {}", e))
                })?,
        );

        let linear_acc_x = f32::from_bits(
            bus.read_u32(200, addresses::IMU_LINEAR_ACCELERATION_X)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!(
                        "TB3 Unable to read imu linear acceleration error: {}",
                        e
                    ))
                })?,
        );

        let linear_acc_y = f32::from_bits(
            bus.read_u32(200, addresses::IMU_LINEAR_ACCELERATION_Y)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!(
                        "TB3 Unable to read imu linear acceleration error: {}",
                        e
                    ))
                })?,
        );

        let linear_acc_z = f32::from_bits(
            bus.read_u32(200, addresses::IMU_LINEAR_ACCELERATION_Z)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!(
                        "TB3 Unable to read imu linear acceleration error: {}",
                        e
                    ))
                })?,
        );

        Ok(IMU {
            orientation: Quaternion {
                x: orientation_x as f64,
                y: orientation_y as f64,
                z: orientation_z as f64,
                w: orientation_w as f64,
            },
            orientation_covariance: [0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64],
            angular_velocity: Vector3 {
                x: velocity_x as f64,
                y: velocity_y as f64,
                z: velocity_z as f64,
            },
            angualar_velocity_covariance: [0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64],
            linear_acceleration: Vector3 {
                x: linear_acc_x as f64,
                y: linear_acc_y as f64,
                z: linear_acc_z as f64,
            },
            linear_acceleration_covariance: [0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64],
        })
    }

    async fn get_magnetic(
        &self,
        bus: &mut dynamixel2::Bus<Vec<u8>, Vec<u8>>,
    ) -> ZFResult<MagneticField> {
        let magnetic_x = f32::from_bits(
            bus.read_u32(200, addresses::IMU_MAGNETIC_X)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!(
                        "TB3 Unable to read imu magnetic field error: {}",
                        e
                    ))
                })?,
        );

        let magnetic_y = f32::from_bits(
            bus.read_u32(200, addresses::IMU_MAGNETIC_Y)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!(
                        "TB3 Unable to read imu magnetic field error: {}",
                        e
                    ))
                })?,
        );

        let magnetic_z = f32::from_bits(
            bus.read_u32(200, addresses::IMU_MAGNETIC_Z)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!(
                        "TB3 Unable to read imu magnetic field error: {}",
                        e
                    ))
                })?,
        );

        Ok(MagneticField {
            magnetic_field: Vector3 {
                x: magnetic_x as f64,
                y: magnetic_y as f64,
                z: magnetic_z as f64,
            },
            magnetic_filed_covariance: [0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64],
        })
    }

    async fn get_joint_states(
        &self,
        bus: &mut dynamixel2::Bus<Vec<u8>, Vec<u8>>,
    ) -> ZFResult<JointState> {
        let position_left = bus
            .read_u32(200, addresses::PRESENT_POSITION_LEFT)
            .await
            .map_err(|e| {
                ZFError::InvalidData(format!(
                    "TB3 Unable to read left motor position error: {}",
                    e
                ))
            })? as u64;

        let position_right = bus
            .read_u32(200, addresses::PRESENT_POSITION_RIGHT)
            .await
            .map_err(|e| {
                ZFError::InvalidData(format!(
                    "TB3 Unable to read right motor position error: {}",
                    e
                ))
            })? as u64;

        let velocity_left = bus
            .read_u32(200, addresses::PRESENT_VELOCITY_LEFT)
            .await
            .map_err(|e| {
                ZFError::InvalidData(format!(
                    "TB3 Unable to read left motor velocity error: {}",
                    e
                ))
            })? as u64;

        let velocity_right = bus
            .read_u32(200, addresses::PRESENT_VELOCITY_RIGHT)
            .await
            .map_err(|e| {
                ZFError::InvalidData(format!(
                    "TB3 Unable to read right motor velocity error: {}",
                    e
                ))
            })? as u64;

        let names = vec![
            "wheel_left_joint".to_string(),
            "wheel_right_joint".to_string(),
        ];
        let velocities = vec![
            RPM_TO_MS * f64::from_bits(velocity_left),
            RPM_TO_MS * f64::from_bits(velocity_right),
        ];
        let positions = vec![
            TICK_TO_RAD * f64::from_bits(position_left),
            TICK_TO_RAD * f64::from_bits(position_right),
        ];

        Ok(JointState {
            name: names,
            position: positions,
            velocity: velocities,
            effort: vec![0f64, 0f64],
        })
    }

    async fn get_sensor_state(
        &self,
        bus: &mut dynamixel2::Bus<Vec<u8>, Vec<u8>>,
    ) -> ZFResult<SensorState> {
        let bumper_fwd_state = bus.read_u8(200, addresses::BUMPER_1).await.map_err(|e| {
            ZFError::InvalidData(format!("TB3 Unable to read bumper position error: {}", e))
        })?;

        let bumper_bwd_state = bus.read_u8(200, addresses::BUMPER_2).await.map_err(|e| {
            ZFError::InvalidData(format!("TB3 Unable to read bumper position error: {}", e))
        })?;

        let mut bumper_push_state = bumper_fwd_state;
        bumper_push_state |= bumper_bwd_state << 1;

        let cliff =
            f32::from_bits(bus.read_u32(200, addresses::IR).await.map_err(|e| {
                ZFError::InvalidData(format!("TB3 Unable to read cliff error: {}", e))
            })?);

        let sonar =
            f32::from_bits(bus.read_u32(200, addresses::SONAR).await.map_err(|e| {
                ZFError::InvalidData(format!("TB3 Unable to read sonar error: {}", e))
            })?);

        let illumination = f32::from_bits(
            bus.read_u32(200, addresses::ILLUMINATION)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!("TB3 Unable to read illumination error: {}", e))
                })?,
        );

        let button_0_state = bus.read_u8(200, addresses::BUTTON_1).await.map_err(|e| {
            ZFError::InvalidData(format!("TB3 Unable to read button 1 state error: {}", e))
        })?;

        let button_1_state = bus.read_u8(200, addresses::BUTTON_2).await.map_err(|e| {
            ZFError::InvalidData(format!("TB3 Unable to read button 2 state error: {}", e))
        })?;

        let mut button_push_state = button_0_state;
        button_push_state |= button_1_state << 1;

        let left_encoder = bus
            .read_u32(200, addresses::PRESENT_POSITION_LEFT)
            .await
            .map_err(|e| {
                ZFError::InvalidData(format!("TB3 Unable to read left encoder error: {}", e))
            })? as i32;

        let right_encoder = bus
            .read_u32(200, addresses::PRESENT_POSITION_RIGHT)
            .await
            .map_err(|e| {
                ZFError::InvalidData(format!("TB3 Unable to read right encoder error: {}", e))
            })? as i32;

        let torque = bus
            .read_u8(200, addresses::MOTOR_TORQUE_ENABLE)
            .await
            .map_err(|e| {
                ZFError::InvalidData(format!(
                    "TB3 Unable to read motor torque enabled error: {}",
                    e
                ))
            })?
            != 0;

        let battery = f32::from_bits(
            bus.read_u32(200, addresses::BATTERY_VOLTAGE)
                .await
                .map_err(|e| {
                    ZFError::InvalidData(format!("TB3 Unable to read battery voltage error: {}", e))
                })?,
        ) * 0.01f32;

        Ok(SensorState {
            bumper: bumper_push_state,
            cliff,
            sonar,
            illumination,
            led: 0,
            button: button_push_state,
            torque,
            left_encoder,
            right_encoder,
            battery,
        })
    }

    async fn write_to_motors(
        &self,
        bus: &mut dynamixel2::Bus<Vec<u8>, Vec<u8>>,
        twist: &Twist,
    ) -> ZFResult<()> {
        bus.write_u32(
            200,
            addresses::CMD_VELOCITY_LINEAR_X,
            ((twist.linear.x * 100.0) as i32) as u32,
        )
        .await
        .map_err(|e| ZFError::InvalidData(format!("TB3 Unable to write to motor error: {}", e)))?;
        bus.write_u32(
            200,
            addresses::CMD_VELOCITY_LINEAR_Y,
            (twist.linear.y as u32) as u32,
        )
        .await
        .map_err(|e| ZFError::InvalidData(format!("TB3 Unable to write to motor error: {}", e)))?;
        bus.write_u32(
            200,
            addresses::CMD_VELOCITY_LINEAR_Z,
            (twist.linear.z as u32) as u32,
        )
        .await
        .map_err(|e| ZFError::InvalidData(format!("TB3 Unable to write to motor error: {}", e)))?;
        bus.write_u32(
            200,
            addresses::CMD_VELOCITY_ANGULAR_X,
            (twist.angular.x as u32) as u32,
        )
        .await
        .map_err(|e| ZFError::InvalidData(format!("TB3 Unable to write to motor error: {}", e)))?;
        bus.write_u32(
            200,
            addresses::CMD_VELOCITY_ANGULAR_Y,
            (twist.angular.y as u32) as u32,
        )
        .await
        .map_err(|e| ZFError::InvalidData(format!("TB3 Unable to write to motor error: {}", e)))?;
        bus.write_u32(
            200,
            addresses::CMD_VELOCITY_ANGULAR_Z,
            ((twist.angular.z * 100.0) as i32) as u32,
        )
        .await
        .map_err(|e| ZFError::InvalidData(format!("TB3 Unable to write to motor error: {}", e)))?;

        Ok(())
    }
}

// Also generated by macro
zenoh_flow::export_operator!(register);

fn register() -> ZFResult<Arc<dyn Operator>> {
    Ok(Arc::new(Tb3) as Arc<dyn Operator>)
}

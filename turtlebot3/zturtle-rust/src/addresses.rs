//
// Copyright (c) 2017, 2020 ADLINK Technology Inc.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ADLINK zenoh team, <zenoh@adlink-labs.tech>
//
#![allow(dead_code)]
pub const MODEL_NUMBER: u16 = 0;
pub const MODEL_INFORMATION: u16 = 2;
pub const FIRMWARE_VERSION: u16 = 6;
pub const ID: u16 = 7;
pub const BAUD_RATE: u16 = 8;

pub const MILLIS: u16 = 10;
pub const MICROS: u16 = 14;

pub const DEVICE_STATUS: u16 = 18;
pub const HEARTBEAT: u16 = 19;

pub const EXTERNAL_LED_1: u16 = 20;
pub const EXTERNAL_LED_2: u16 = 21;
pub const EXTERNAL_LED_3: u16 = 22;
pub const EXTERNAL_LED_4: u16 = 23;

pub const BUTTON_1: u16 = 26;
pub const BUTTON_2: u16 = 27;

pub const BUMPER_1: u16 = 28;
pub const BUMPER_2: u16 = 29;

pub const ILLUMINATION: u16 = 30;
pub const IR: u16 = 34;
pub const SONAR: u16 = 38;

pub const BATTERY_VOLTAGE: u16 = 42;
pub const BATTERY_PERCENTAGE: u16 = 46;

pub const SOUND: u16 = 50;

pub const IMU_RE_CALIBRATION: u16 = 59;

pub const IMU_ANGULAR_VELOCITY_X: u16 = 60;
pub const IMU_ANGULAR_VELOCITY_Y: u16 = 64;
pub const IMU_ANGULAR_VELOCITY_Z: u16 = 68;
pub const IMU_LINEAR_ACCELERATION_X: u16 = 72;
pub const IMU_LINEAR_ACCELERATION_Y: u16 = 76;
pub const IMU_LINEAR_ACCELERATION_Z: u16 = 80;
pub const IMU_MAGNETIC_X: u16 = 84;
pub const IMU_MAGNETIC_Y: u16 = 88;
pub const IMU_MAGNETIC_Z: u16 = 92;
pub const IMU_ORIENTATION_W: u16 = 96;
pub const IMU_ORIENTATION_X: u16 = 100;
pub const IMU_ORIENTATION_Y: u16 = 104;
pub const IMU_ORIENTATION_Z: u16 = 108;

pub const PRESENT_CURRENT_LEFT: u16 = 120;
pub const PRESENT_CURRENT_RIGHT: u16 = 124;
pub const PRESENT_VELOCITY_LEFT: u16 = 128;
pub const PRESENT_VELOCITY_RIGHT: u16 = 132;
pub const PRESENT_POSITION_LEFT: u16 = 136;
pub const PRESENT_POSITION_RIGHT: u16 = 140;

pub const MOTOR_TORQUE_ENABLE: u16 = 149;

pub const CMD_VELOCITY_LINEAR_X: u16 = 150;
pub const CMD_VELOCITY_LINEAR_Y: u16 = 154;
pub const CMD_VELOCITY_LINEAR_Z: u16 = 158;
pub const CMD_VELOCITY_ANGULAR_X: u16 = 162;
pub const CMD_VELOCITY_ANGULAR_Y: u16 = 166;
pub const CMD_VELOCITY_ANGULAR_Z: u16 = 170;

pub const PROFILE_ACCELERATION_LEFT: u16 = 174;
pub const PROFILE_ACCELERATION_RIGHT: u16 = 178;

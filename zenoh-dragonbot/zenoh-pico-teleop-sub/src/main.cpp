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


#include "config.hpp"
#include <iostream>
#include <unistd.h>

/*******************************************************************************
* Zenoh Session initialization
*******************************************************************************/
z_owned_session_t zenohInit()
{
    z_owned_config_t config = zp_config_default();
    zp_config_insert(z_config_loan(&config), Z_CONFIG_MODE_KEY, z_string_make(MODE));
    if (strcmp(PEER, "") != 0) {
        zp_config_insert(z_config_loan(&config), Z_CONFIG_PEER_KEY, z_string_make(PEER));
    }
    z_owned_session_t s = z_open(z_config_move(&config));
    return s;
}


int main (int argc, char *argv[]) {
    z_owned_session_t z;

    z = zenohInit();
    if (!z_session_check(&z))
    {
        std::cout << "Error establishing zenoh session!" << std::endl;
        return -1;
    }



    zp_start_read_task(z_session_loan(&z));
    zp_start_lease_task(z_session_loan(&z));

    unsigned long rid = 0;

    // Registering Subscriptions

    z_owned_closure_sample_t callback_sensor_state = z_closure_sample(sensorStateCallback, NULL, NULL);
    sensor_state_sub = z_declare_subscriber(z_session_loan(&z), z_keyexpr(SENSOR_STATE), z_closure_sample_move(&callback_sensor_state), NULL);
    if (!z_subscriber_check(&sensor_state_sub)) {
        std::cout << "Error declaring sensor state subscriber!" << std::endl;
        return -1;
    }

    z_owned_closure_sample_t callback_firmware_version = z_closure_sample(firmwareVersionCallback, NULL, NULL);
    firmware_version_sub = z_declare_subscriber(z_session_loan(&z), z_keyexpr(FIRMWARE_VERSION), z_closure_sample_move(&callback_firmware_version), NULL);
    if (!z_subscriber_check(&firmware_version_sub)) {
        std::cout << "Error declaring firmware version subscriber!" << std::endl;
        return -1;
    }

    z_owned_closure_sample_t callback_imu = z_closure_sample(firmwareVersionCallback, NULL, NULL);
    imu_sub = z_declare_subscriber(z_session_loan(&z), z_keyexpr(IMU), z_closure_sample_move(&callback_imu), NULL);
    if (!z_subscriber_check(&imu_sub)) {
        std::cout << "Error declaring IMU subscriber!" << std::endl;
        return -1;
    }

    z_owned_closure_sample_t callback_magnetic_field = z_closure_sample(magneticFieldCallback, NULL, NULL);
    magnetic_field_sub = z_declare_subscriber(z_session_loan(&z), z_keyexpr(FIRMWARE_VERSION), z_closure_sample_move(&callback_magnetic_field), NULL);
    if (!z_subscriber_check(&magnetic_field_sub)) {
        std::cout << "Error declaring magnetic field subscriber!" << std::endl;
        return -1;
    }

    z_owned_closure_sample_t callback_battery_state = z_closure_sample(batteryStateCallback, NULL, NULL);
    battery_state_sub = z_declare_subscriber(z_session_loan(&z), z_keyexpr(FIRMWARE_VERSION), z_closure_sample_move(&callback_battery_state), NULL);
    if (!z_subscriber_check(&battery_state_sub)) {
        std::cout << "Error declaring battery state subscriber!" << std::endl;
        return -1;
    }

    while (true) { sleep(10); }

}





/*******************************************************************************
* Subscribers callbacks (print the received message)
*******************************************************************************/
void sensorStateCallback(const z_sample_t *sample, void *arg) {
    turtlebot3_msgs::SensorState sensor_state_msg;
    sensor_state_msg.deserialize((unsigned char*)sample->payload.start);
}

void firmwareVersionCallback(const z_sample_t *sample, void *arg) {
    turtlebot3_msgs::VersionInfo version_info_msg;
    version_info_msg.deserialize((unsigned char*)sample->payload.start);

    std::cout << "### Version ###" << std::endl <<
        "HW Version: " << version_info_msg.hardware << std::endl <<
        "FW Version: " << version_info_msg.firmware << std::endl <<
        "SW Version: " << version_info_msg.software << std::endl <<
        "###############" << std::endl <<
        std::flush;

}

void imuCallback(const z_sample_t *sample, void *arg) {
    sensor_msgs::Imu imu_msg;
    imu_msg.deserialize((unsigned char*)sample->payload.start);

    std::cout << "### IMU ###" << std::endl <<
        "Linear Acceleration X: " << imu_msg.linear_acceleration.x << std::endl <<
        "Linear Acceleration Y: " << imu_msg.linear_acceleration.y << std::endl <<
        "Linear Acceleration Z: " << imu_msg.linear_acceleration.z << std::endl <<
        "###########" << std::endl <<
        std::flush;
}

void cmdVelRc100Callback(const z_sample_t *sample, void *arg) {
    geometry_msgs::Twist cmd_vel_rc100_msg;
    cmd_vel_rc100_msg.deserialize((unsigned char*)sample->payload.start);
}

void odomCallback(const z_sample_t *sample, void *arg) {
    nav_msgs::Odometry odom_msg;
    odom_msg.deserialize((unsigned char*)sample->payload.start);
}

void jointStatesCallback(const z_sample_t *sample, void *arg) {
    sensor_msgs::JointState joint_states_msg;
    joint_states_msg.deserialize((unsigned char*)sample->payload.start);
}

void batteryStateCallback(const z_sample_t *sample, void *arg) {
    sensor_msgs::BatteryState battery_state_msg;
    battery_state_msg.deserialize((unsigned char*)sample->payload.start);


    std::cout << "### Battery ###" << std::endl <<
        "Voltage: " << battery_state_msg.voltage << std::endl <<
        "Current " << battery_state_msg.current << std::endl <<
        "Charge: " << battery_state_msg.charge << std::endl <<
        "Capacity: " << battery_state_msg.capacity << std::endl <<
        "Design Capacity " << battery_state_msg.design_capacity << std::endl <<
        "Percentage: " << battery_state_msg.percentage << std::endl <<
        "Power Supply Status: " << battery_state_msg.power_supply_status << std::endl <<
        "Power Supply Health: " << battery_state_msg.power_supply_health << std::endl <<
        "Power Supply Technology: " << battery_state_msg.power_supply_technology << std::endl <<
        "Present: " << battery_state_msg.present << std::endl <<
        "Cell Voltage Length: " << battery_state_msg.cell_voltage_length << std::endl <<
        "ST Cell Voltage: " << battery_state_msg.st_cell_voltage << std::endl <<
        "Cell Voltage: " << battery_state_msg.cell_voltage << std::endl <<
        "Location: " << battery_state_msg.location << std::endl <<
        "Serial Number: " << battery_state_msg.serial_number << std::endl <<
        "##############" << std::endl <<
        std::flush;
}

void magneticFieldCallback(const z_sample_t *sample, void *arg) {
    sensor_msgs::MagneticField mag_msg;
    mag_msg.deserialize((unsigned char*)sample->payload.start);

    std::cout << "### Magnetic ###" << std::endl <<
        "Magnetic X: " << mag_msg.magnetic_field.x << std::endl <<
        "Magnetic Y: " << mag_msg.magnetic_field.y << std::endl <<
        "Magnetic Z: " << mag_msg.magnetic_field.z << std::endl <<
        "################" << std::endl <<
        std::flush;
}

void broadcastTfCallback(const z_sample_t *sample, void *arg) {
    (void) sample;
    (void) arg;
}
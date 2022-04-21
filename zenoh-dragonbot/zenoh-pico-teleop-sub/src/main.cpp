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
zn_session_t *zenohInit()
{
    zn_properties_t *config = zn_config_default();
    zn_properties_insert(config, ZN_CONFIG_MODE_KEY, z_string_make(MODE));
    if (strcmp(PEER, "") != 0)
        zn_properties_insert(config, ZN_CONFIG_PEER_KEY, z_string_make(PEER));

    zn_session_t *s = zn_open(config);

    return s;
}


int main (int argc, char *argv[]) {
    zn_session_t *zn = NULL;

    zn = zenohInit();
    if (zn == NULL)
    {
        std::cout << "Error establishing zenoh session!" << std::endl;
        return -1;
    }



    znp_start_read_task(zn);
    znp_start_lease_task(zn);

    unsigned long rid = 0;

    // Registering ResKeys Subscriptions

    rid = zn_declare_resource(zn, zn_rname(CMD_VEL));
    rk_cmd_vel = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *rk_cmd_vel = zn_rid(rid);

    rid = zn_declare_resource(zn, zn_rname(SOUND));
    rk_sound = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *rk_sound = zn_rid(rid);

    rid = zn_declare_resource(zn, zn_rname(MOTOR_POWER));
    rk_motor_power = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *rk_motor_power = zn_rid(rid);

    rid = zn_declare_resource(zn, zn_rname(RESET));
    rk_reset = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *rk_reset = zn_rid(rid);

    // Registering ResKeys Publications

    rid = zn_declare_resource(zn, zn_rname(SENSOR_STATE));
    rk_sensor_state = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *rk_sensor_state = zn_rid(rid);

    rid = zn_declare_resource(zn, zn_rname(FIRMWARE_VERSION));
    rk_firmware_version = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *rk_firmware_version = zn_rid(rid);

    rid = zn_declare_resource(zn, zn_rname(IMU));
    rk_imu = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *rk_imu = zn_rid(rid);

    rid = zn_declare_resource(zn, zn_rname(CMD_VEL_RC100));
    rk_cmd_vel_rc100 = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *rk_cmd_vel_rc100 = zn_rid(rid);

    rid = zn_declare_resource(zn, zn_rname(ODOM));
    rk_odom = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *rk_odom = zn_rid(rid);

    rid = zn_declare_resource(zn, zn_rname(JOINT_STATES));
    rk_joint_states = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *rk_joint_states = zn_rid(rid);

    rid = zn_declare_resource(zn, zn_rname(BATTERY_STATE));
    rk_battery_state = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *rk_battery_state = zn_rid(rid);

    rid = zn_declare_resource(zn, zn_rname(MAGNETIC_FIELD));
    rk_magnetic_field = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *rk_magnetic_field = zn_rid(rid);

    rid = zn_declare_resource(zn, zn_rname(BROADCAST_TF));
    rk_broadcast_tf = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *rk_broadcast_tf = zn_rid(rid);

    // Registering subscriber

    firmaware_version_sub = zn_declare_subscriber(zn, *rk_firmware_version, zn_subinfo_default(), firmwareVersionCallback, NULL);
    imu_sub = zn_declare_subscriber(zn, *rk_imu, zn_subinfo_default(), imuCallback, NULL);
    magnetic_field_sub = zn_declare_subscriber(zn, *rk_magnetic_field, zn_subinfo_default(), magneticFieldCallback, NULL);
    battery_state_sub =  zn_declare_subscriber(zn, *rk_battery_state, zn_subinfo_default(), batteryStateCallback, NULL);

    while (true) { sleep(10); }

}





/*******************************************************************************
* Subscribers callbacks (print the received message)
*******************************************************************************/
void sensorStateCallback(const zn_sample_t *sample, const void *arg) {
    turtlebot3_msgs::SensorState sensor_state_msg;
    sensor_state_msg.deserialize((unsigned char*)sample->value.val);
}

void firmwareVersionCallback(const zn_sample_t *sample, const void *arg) {
    turtlebot3_msgs::VersionInfo version_info_msg;
    version_info_msg.deserialize((unsigned char*)sample->value.val);

    std::cout << "### Version ###" << std::endl <<
        "HW Version: " << version_info_msg.hardware << std::endl <<
        "FW Version: " << version_info_msg.firmware << std::endl <<
        "SW Version: " << version_info_msg.software << std::endl <<
        "###############" << std::endl <<
        std::flush;

}

void imuCallback(const zn_sample_t *sample, const void *arg) {
    sensor_msgs::Imu imu_msg;
    imu_msg.deserialize((unsigned char*)sample->value.val);

    std::cout << "### IMU ###" << std::endl <<
        "Linear Acceleration X: " << imu_msg.linear_acceleration.x << std::endl <<
        "Linear Acceleration Y: " << imu_msg.linear_acceleration.y << std::endl <<
        "Linear Acceleration Z: " << imu_msg.linear_acceleration.z << std::endl <<
        "###########" << std::endl <<
        std::flush;
}

void cmdVelRc100Callback(const zn_sample_t *sample, const void *arg) {
    geometry_msgs::Twist cmd_vel_rc100_msg;
    cmd_vel_rc100_msg.deserialize((unsigned char*)sample->value.val);
}

void odomCallback(const zn_sample_t *sample, const void *arg) {
    nav_msgs::Odometry odom_msg;
    odom_msg.deserialize((unsigned char*)sample->value.val);
}

void jointStatesCallback(const zn_sample_t *sample, const void *arg) {
    sensor_msgs::JointState joint_states_msg;
    joint_states_msg.deserialize((unsigned char*)sample->value.val);
}

void batteryStateCallback(const zn_sample_t *sample, const void *arg) {
    sensor_msgs::BatteryState battery_state_msg;
    battery_state_msg.deserialize((unsigned char*)sample->value.val);


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

void magneticFieldCallback(const zn_sample_t *sample, const void *arg) {
    sensor_msgs::MagneticField mag_msg;
    mag_msg.deserialize((unsigned char*)sample->value.val);

    std::cout << "### Magnetic ###" << std::endl <<
        "Magnetic X: " << mag_msg.magnetic_field.x << std::endl <<
        "Magnetic Y: " << mag_msg.magnetic_field.y << std::endl <<
        "Magnetic Z: " << mag_msg.magnetic_field.z << std::endl <<
        "################" << std::endl <<
        std::flush;
}

void broadcastTfCallback(const zn_sample_t *sample, const void *arg) {
    ;
}

/*******************************************************************************
* Publication functions
*******************************************************************************/
void publishCmdVel(zn_session_t *zn, geometry_msgs::Twist cmd_vel_msg) {
    unsigned char buf[1024];
    int size = cmd_vel_msg.serialize(buf);
    zn_write(zn, *rk_cmd_vel, (const uint8_t *)buf, size);
}


void publishSound(zn_session_t *zn, turtlebot3_msgs::Sound sound_msg) {
    unsigned char buf[1024];
    int size = sound_msg.serialize(buf);
    zn_write(zn, *rk_sound, (const uint8_t *)buf, size);
}

void publishMotorPower(zn_session_t *zn, std_msgs::Bool motor_power_msg) {
    unsigned char buf[1024];
    int size = motor_power_msg.serialize(buf);
    zn_write(zn, *rk_motor_power, (const uint8_t *)buf, size);
}

void publishReset(zn_session_t *zn, std_msgs::Empty reset_msg) {
    unsigned char buf[1024];
    int size = reset_msg.serialize(buf);
    zn_write(zn, *rk_reset, (const uint8_t *)buf, size);
}
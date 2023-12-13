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

#ifndef TURTLEBOT3_CONFIG_H_
#define TURTLEBOT3_CONFIG_H_

#include <TurtleBot3_ROS2.h>
#include <std_msgs/Bool.h>
#include <std_msgs/Empty.h>
#include <std_msgs/Int32.h>
#include <sensor_msgs/JointState.h>
#include <sensor_msgs/Imu.h>
#include <sensor_msgs/MagneticField.h>
#include <sensor_msgs/BatteryState.h>
#include <geometry_msgs/Vector3.h>
#include <tf/tf.h>
#include <tf/transform_broadcaster.h>
#include <nav_msgs/Odometry.h>

#include <turtlebot3_msgs/SensorState.h>
#include <turtlebot3_msgs/Sound.h>
#include <turtlebot3_msgs/VersionInfo.h>
#include <tf/tfMessage.h>
#include <time.h>

#include <RTOS.h>

extern "C"
{
#include "zenoh-pico.h"
}

/*******************************************************************************
 * WiFi and Zenoh configuration
 *******************************************************************************/

#define SSID "NAME"
#define PASS "PASSWORD"

#define MODE "client"
#define CONNECT "tcp/192.168.86.57:7447"

/*******************************************************************************
 * Robot Definitions
 *******************************************************************************/
#define NAME "Burger"

#define MODEL_INFO 1
#define MAX_RPM 61
#define WHEEL_RADIUS 0.033      // meter
#define WHEEL_SEPARATION 0.160  // meter (BURGER : 0.160, WAFFLE : 0.287)
#define TURNING_RADIUS 0.080    // meter (BURGER : 0.080, WAFFLE : 0.1435)
#define ROBOT_RADIUS 0.105      // meter (BURGER : 0.105, WAFFLE : 0.220)
#define ENCODER_MIN -2147483648 // raw
#define ENCODER_MAX 2147483648  // raw

#define MAX_LINEAR_VELOCITY (WHEEL_RADIUS * 2 * 3.14159265359 * 61 / 60) // m/s  (BURGER : 61[rpm], WAFFLE : 77[rpm])
#define MAX_ANGULAR_VELOCITY (MAX_LINEAR_VELOCITY / TURNING_RADIUS)      // rad/s

#define MIN_LINEAR_VELOCITY -MAX_LINEAR_VELOCITY
#define MIN_ANGULAR_VELOCITY -MAX_ANGULAR_VELOCITY

#define FIRMWARE_VER "0.0.2"

#define CONTROL_MOTOR_SPEED_FREQUENCY 30        // hz
#define CONTROL_MOTOR_TIMEOUT 500               // ms
#define IMU_PUBLISH_FREQUENCY 200               // hz
#define CMD_VEL_PUBLISH_FREQUENCY 30            // hz
#define DRIVE_INFORMATION_PUBLISH_FREQUENCY 30  // hz
#define VERSION_INFORMATION_PUBLISH_FREQUENCY 1 // hz
#define DEBUG_LOG_FREQUENCY 10                  // hz

#define WHEEL_NUM 2

#define LEFT 0
#define RIGHT 1

#define ROS1_LINEAR 0
#define ROS1_ANGULAR 1

#define DEG2RAD(x) (x * 0.01745329252) // *PI/180
#define RAD2DEG(x) (x * 57.2957795131) // *180/PI

#define TICK2RAD 0.001533981 // 0.087890625[deg] * 3.14159265359 / 180 = 0.001533981f

#define TEST_DISTANCE 0.300 // meter
#define TEST_RADIAN 3.14    // 180 degree

// #define DEBUG
#define DEBUG_SERIAL SerialBT2

#define ZENOH_DEBUG 3

typedef struct TB3ModelInfo
{
    const char *model_str;
    uint32_t model_info;
    uint16_t model_motor_rpm;
    float wheel_radius;
    float wheel_separation;
    float turning_radius;
    float robot_radius;
} TB3ModelInfo;

static const TB3ModelInfo burger_info = {
    NAME,
    MODEL_INFO,
    MAX_RPM,
    WHEEL_RADIUS,
    WHEEL_SEPARATION,
    TURNING_RADIUS,
    ROBOT_RADIUS,
};

/*******************************************************************************
 * Zenoh Key Expression Subscriptions
 *******************************************************************************/
#define CMD_VEL "rt/cmd_vel"
#define SOUND "rt/sound"
#define MOTOR_POWER "rt/motor_power"
#define RESET "rt/reset"

/*******************************************************************************
 * Zenoh Key Expression Publications
 *******************************************************************************/
#define SENSOR_STATE "rt/sensor_state"
#define FIRMWARE_VERSION "rt/firmware_version"
#define IMU "rt/imu"
#define CMD_VEL_RC100 "rt/cmd_vel_rc100"
#define ODOM "rt/odom"
#define JOINT_STATES "rt/joint_states"
#define BATTERY_STATE "rt/battery_state"
#define MAGNETIC_FIELD "rt/magnetic_field"
#define BROADCAST_TF "rt/tf"

/*******************************************************************************
 * Zenoh Publishers
 *******************************************************************************/
z_owned_publisher_t pub_sensor_state;
z_owned_publisher_t pub_firmware_version;
z_owned_publisher_t pub_imu;
z_owned_publisher_t pub_cmd_vel_rc100;
z_owned_publisher_t pub_odom;
z_owned_publisher_t pub_joint_states;
z_owned_publisher_t pub_battery_state;
z_owned_publisher_t pub_magnetic_field;
z_owned_publisher_t pub_broadcast_tf;

/*******************************************************************************
 * Zenoh Subscribers
 *******************************************************************************/
z_owned_subscriber_t sub_cmd_vel;
z_owned_subscriber_t sub_sound;
z_owned_subscriber_t sub_motor_power;
z_owned_subscriber_t sub_reset;

/*******************************************************************************
 * Zenoh Session and global variables
 *******************************************************************************/
z_owned_session_t s;
bool led_1_status = false;
bool led_2_status = true;
bool led_3_status = true;
bool led_4_status = true;
unsigned char buf[1024];

/*******************************************************************************
 * Subscribers messages
 *******************************************************************************/
geometry_msgs::Twist cmd_vel_msg;
turtlebot3_msgs::Sound sound_msg;
std_msgs::Bool motor_power_msg;
std_msgs::Empty reset_msg;

/*******************************************************************************
 * Publisher messages
 *******************************************************************************/
// Bumpers, cliffs, buttons, encoders, battery of Turtlebot3
turtlebot3_msgs::SensorState sensor_state_msg;

// Version information of Turtlebot3
turtlebot3_msgs::VersionInfo version_info_msg;

// IMU of Turtlebot3
sensor_msgs::Imu imu_msg;

// Command velocity of Turtlebot3 using RC100 remote controller
geometry_msgs::Twist cmd_vel_rc100_msg;

// Odometry of Turtlebot3
nav_msgs::Odometry odom_msg;

// Joint(Dynamixel) state of Turtlebot3
sensor_msgs::JointState joint_states_msg;

// Battey state of Turtlebot3
sensor_msgs::BatteryState battery_state_msg;

// Magnetic field
sensor_msgs::MagneticField mag_msg;

/*******************************************************************************
 * Subscribers callbacks
 *******************************************************************************/
void resetCallback(const z_sample_t *sample, void *arg);
void motorPowerCallback(const z_sample_t *sample, void *arg);
void soundCallback(const z_sample_t *sample, void *arg);
void commandVelocityCallback(const z_sample_t *sample, void *arg);

/*******************************************************************************
 * Publication functions
 *******************************************************************************/
void publishImuMsg(z_publisher_t *pub);
void publishMagMsg(z_publisher_t *pub);
void publishSensorStateMsg(z_publisher_t *pub);
void publishVersionInfoMsg(z_publisher_t *pub);
void publishBatteryStateMsg(z_publisher_t *pub);
void publishDriveInformation(z_publisher_t *pub_odom, z_publisher_t *pub_tf, z_publisher_t *pub_js);
void sendTransform(z_publisher_t *pub);

/*******************************************************************************
 * Helper functions
 *******************************************************************************/
bool calcOdometry(double diff_time);
void updateOdometry(void);
void initOdom(void);
void updateTF(geometry_msgs::TransformStamped &odom_tf);
void updateJointStates(void);
void initJointStates(void);
void updateMotorInfo(int32_t left_tick, int32_t right_tick);
void updateGoalVelocity(void);
void initTimes(void);

/*******************************************************************************
 * FIXME: Thread handlers
 *******************************************************************************/
osThreadId thread_id_update;
osThreadId thread_id_read;
osThreadId thread_id_lease;

SemaphoreHandle_t handle;

/*******************************************************************************
 * FIXME: Thread functions
 *******************************************************************************/
static void run_read_task(void const *args);
static void run_lease_task(void const *args);
static void run_update(void const *args);

/*******************************************************************************
 * ROS Parameter
 *******************************************************************************/
char get_prefix[10];
char *get_tf_prefix = get_prefix;

char odom_header_frame_id[30];
char odom_child_frame_id[30];

char imu_frame_id[30];
char mag_frame_id[30];

char joint_state_header_frame_id[30];

ros::Time fake_time = ros::Time(0, 0);

/*******************************************************************************
 * Transform Broadcaster
 *******************************************************************************/
// TF of Turtlebot3
geometry_msgs::TransformStamped odom_tf;
tf::TransformBroadcaster tf_broadcaster; // publisher on "/tf"

/*******************************************************************************
 * Declaration for motor
 *******************************************************************************/
Turtlebot3MotorDriver motor_driver;

/*******************************************************************************
 * Calculation for odometry
 *******************************************************************************/
bool init_encoder = true;
int32_t last_diff_tick[WHEEL_NUM] = {0, 0};
double last_rad[WHEEL_NUM] = {0.0, 0.0};

/*******************************************************************************
 * Update Joint State
 *******************************************************************************/
double last_velocity[WHEEL_NUM] = {0.0, 0.0};

/*******************************************************************************
 * Declaration for sensors
 *******************************************************************************/
Turtlebot3Sensor sensors;

/*******************************************************************************
 * Declaration for controllers
 *******************************************************************************/
Turtlebot3Controller controllers;
static float max_linear_velocity, min_linear_velocity;
static float max_angular_velocity, min_angular_velocity;
float zero_velocity[VelocityType::TYPE_NUM_MAX] = {0.0, 0.0};
float goal_velocity[VelocityType::TYPE_NUM_MAX] = {0.0, 0.0};
float goal_velocity_from_cmd[VelocityType::TYPE_NUM_MAX] = {0.0, 0.0};

/*******************************************************************************
 * Declaration for diagnosis
 *******************************************************************************/
Turtlebot3Diagnosis diagnosis;

/*******************************************************************************
 * Declaration for SLAM and navigation
 *******************************************************************************/
unsigned long prev_update_time;
float odom_pose[3];
double odom_vel[3];

/*******************************************************************************
 * Declaration for Battery
 *******************************************************************************/
bool setup_end = false;
uint8_t battery_state = 0;

/*******************************************************************************
 * SoftwareTimer of Turtlebot3
 *******************************************************************************/
static uint32_t tTime[10];

#endif // TURTLEBOT3_CONFIG_H_
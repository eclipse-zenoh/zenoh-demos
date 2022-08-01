

#ifndef CONFIG_H_
#define CONFIG_H_


#include "turtlebot3_ros_lib/std_msgs/Bool.h"
#include "turtlebot3_ros_lib/std_msgs/Empty.h"
#include "turtlebot3_ros_lib/std_msgs/Int32.h"
#include "turtlebot3_ros_lib/sensor_msgs/JointState.h"
#include "turtlebot3_ros_lib/sensor_msgs/Imu.h"
#include "turtlebot3_ros_lib/sensor_msgs/MagneticField.h"
#include "turtlebot3_ros_lib/sensor_msgs/BatteryState.h"
#include "turtlebot3_ros_lib/geometry_msgs/Vector3.h"
#include "turtlebot3_ros_lib/tf/tf.h"
#include "turtlebot3_ros_lib/tf/transform_broadcaster.h"
#include "turtlebot3_ros_lib/nav_msgs/Odometry.h"
#include "turtlebot3_ros_lib/turtlebot3_msgs/SensorState.h"
#include "turtlebot3_ros_lib/turtlebot3_msgs/Sound.h"
#include "turtlebot3_ros_lib/turtlebot3_msgs/VersionInfo.h"
#include "turtlebot3_ros_lib/tf/tfMessage.h"


extern "C"
{
    #include "zenoh-pico.h"
}

#define MODE "client"
#define PEER "tcp/192.168.86.12:7447"


/*******************************************************************************
* Zenoh Key Expression Publications
*******************************************************************************/
#define CMD_VEL "rt/cmd_vel"
#define SOUND "rt/sound"
#define MOTOR_POWER "rt/motor_power"
#define RESET "rt/reset"

/*******************************************************************************
* Zenoh Key Expression Subscriptions
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


#define ROBOT_LINEAR_ACCELERATION_X "paris/dragonbot/acceleration/linear/x"
#define ROBOT_LINEAR_ACCELERATION_Y "paris/dragonbot/acceleration/linear/y"
#define ROBOT_LINEAR_ACCELERATION_Z "paris/dragonbot/acceleration/linear/z"

#define ROBOT_ANGULAR_VELOCITY_X "paris/dragonbot/velocity/angular/x"
#define ROBOT_ANGULAR_VELOCITY_Y "paris/dragonbot/velocity/angular/y"
#define ROBOT_ANGULAR_VELOCITY_Z "paris/dragonbot/velocity/angular/z"

#define BATTERY_VOLTAGE "paris/dragonbot/battery/voltage"


/*******************************************************************************
* Zenoh Subscribers
*******************************************************************************/
z_owned_subscriber_t sensor_state_sub;
z_owned_subscriber_t firmware_version_sub;
z_owned_subscriber_t imu_sub;
z_owned_subscriber_t cmd_vel_rc100_sub;
z_owned_subscriber_t odom_sub;
z_owned_subscriber_t joint_states_sub;
z_owned_subscriber_t battery_state_sub;
z_owned_subscriber_t magnetic_field_sub;
z_owned_subscriber_t broadcast_tf_sub;

/*******************************************************************************
* Subscribers callbacks
*******************************************************************************/
void sensorStateCallback(const z_sample_t *sample, void *arg);
void firmwareVersionCallback(const z_sample_t *sample, void *arg);
void imuCallback(const z_sample_t *sample, void *arg);
void cmdVelRc100Callback(const z_sample_t *sample, void *arg);
void odomCallback(const z_sample_t *sample, void *arg);
void jointStatesCallback(const z_sample_t *sample, void *arg);
void batteryStateCallback(const z_sample_t *sample, void *arg);
void magneticFieldCallback(const z_sample_t *sample, void *arg);
void broadcastTfCallback(const z_sample_t *sample, void *arg);



#endif // CONFIG_H_


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
#define CMD_VEL "/rt/cmd_vel"
#define SOUND "/rt/sound"
#define MOTOR_POWER "/rt/motor_power"
#define RESET "/rt/reset"

/*******************************************************************************
* Zenoh Key Expression Subscriptions
*******************************************************************************/
#define SENSOR_STATE "/rt/sensor_state"
#define FIRMWARE_VERSION "/rt/firmware_version"
#define IMU "/rt/imu"
#define CMD_VEL_RC100 "/rt/cmd_vel_rc100"
#define ODOM "/rt/odom"
#define JOINT_STATES "/rt/joint_states"
#define BATTERY_STATE "/rt/battery_state"
#define MAGNETIC_FIELD "/rt/magnetic_field"
#define BROADCAST_TF "/rt/tf"



/*******************************************************************************
* Zenoh ResKey Publications
*******************************************************************************/
zn_reskey_t *rk_cmd_vel = NULL;
zn_reskey_t *rk_sound = NULL;
zn_reskey_t *rk_motor_power = NULL;
zn_reskey_t *rk_reset = NULL;

/*******************************************************************************
* Zenoh ResKey Subscriptions
*******************************************************************************/
zn_reskey_t *rk_sensor_state = NULL;
zn_reskey_t *rk_firmware_version = NULL;
zn_reskey_t *rk_imu = NULL;
zn_reskey_t *rk_cmd_vel_rc100 = NULL;
zn_reskey_t *rk_odom = NULL;
zn_reskey_t *rk_joint_states = NULL;
zn_reskey_t *rk_battery_state = NULL;
zn_reskey_t *rk_magnetic_field = NULL;
zn_reskey_t *rk_broadcast_tf = NULL;

/*******************************************************************************
* Zenoh Subscribers
*******************************************************************************/
zn_subscriber_t *sensor_state_sub = NULL;
zn_subscriber_t *firmaware_version_sub = NULL;
zn_subscriber_t *imu_sub = NULL;
zn_subscriber_t *cmd_vel_rc100_sub = NULL;
zn_subscriber_t *odom_sub = NULL;
zn_subscriber_t *joint_states_sub = NULL;
zn_subscriber_t *battery_state_sub = NULL;
zn_subscriber_t *magnetic_field_sub = NULL;
zn_subscriber_t *broadcast_tf_sub = NULL;

/*******************************************************************************
* Subscribers callbacks
*******************************************************************************/
void sensorStateCallback(const zn_sample_t *sample, const void *arg);
void firmwareVersionCallback(const zn_sample_t *sample, const void *arg);
void imuCallback(const zn_sample_t *sample, const void *arg);
void cmdVelRc100Callback(const zn_sample_t *sample, const void *arg);
void odomCallback(const zn_sample_t *sample, const void *arg);
void jointStatesCallback(const zn_sample_t *sample, const void *arg);
void batteryStateCallback(const zn_sample_t *sample, const void *arg);
void magneticFieldCallback(const zn_sample_t *sample, const void *arg);
void broadcastTfCallback(const zn_sample_t *sample, const void *arg);

/*******************************************************************************
* Publication functions
*******************************************************************************/
void publishCmdVel(zn_session_t*, geometry_msgs::Twist);
void publishSound(zn_session_t*, turtlebot3_msgs::Sound);
void publishMotorPower(zn_session_t*, std_msgs::Bool);
void publishReset(zn_session_t*, std_msgs::Empty);


#endif // CONFIG_H_
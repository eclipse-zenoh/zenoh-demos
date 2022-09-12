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

#include <SPI.h>
#include <WiFi101.h>

#include "turtlebot3_config.h"

/*******************************************************************************
* Interrupt functions needed for WiFi
*******************************************************************************/
extern "C" void attachInterruptMultiArch(uint32_t pin, void *chip_isr, uint32_t mode)
{
    void (*_c)(void) = (void(*)(void))(chip_isr);
    attachInterrupt(pin, _c, mode);
}

extern "C" void detachInterruptMultiArch(uint32_t pin)
{
    detachInterrupt(pin);
}

/*******************************************************************************
* WiFi setup
*******************************************************************************/
int wifiInit()
{
    // Required to remap interrupt pin with opencr
    WiFi.setPins(10, digitalPinToInterrupt(7), 5, -1);

    if (WiFi.status() == WL_NO_SHIELD)
        return -1;

    WiFi.begin(SSID, PASS);
}

/*******************************************************************************
* Thread functions
*******************************************************************************/
static void run_loop(void const *argument)
{
  (void) argument;

  for(;;)
  {
    while (xSemaphoreTake(handle, ( TickType_t ) configTICK_RATE_HZ ) != pdTRUE );
    Serial.println("Thread Loop");
    loop();
    xSemaphoreGive(handle);
    delay(10);
  }
}

static void run_read_task(void const *argument)
{
  (void) argument;

  for(;;)
  {
    while (xSemaphoreTake(handle, ( TickType_t ) configTICK_RATE_HZ ) != pdTRUE );
    Serial.println("Thread Read");
    zp_read(z_session_loan(&s), NULL);
    xSemaphoreGive(handle);
    delay(10);
  }
}

static void run_lease_task(void const *argument)
{
  (void) argument;

  for(;;)
  {
    while (xSemaphoreTake(handle, ( TickType_t ) configTICK_RATE_HZ ) != pdTRUE );
    Serial.println("Thread Keep Alive");
    zp_send_keep_alive(z_session_loan(&s), NULL);
    xSemaphoreGive(handle);
    delay(1000);
  }
}

void setup()
{
    // Initialize Serial for debug
    Serial.begin(115200);

    // Initialize motors drivers
    int ret = motor_driver.init();
    if (ret == false)
    {
        Serial.println("Failing to initialize the motor drivers");
        while(true);
    }

    // Setting for IMU
    ret = sensors.init();
    if (ret == false)
    {
        Serial.println("Failing to initialize the sensors");
        while(true);
    }

    // // Init diagnosis
    // ret = diagnosis.init();
    // if (ret == false)
    // {
    //     Serial.println("Failing to initialize the diagnosis");
    //     while(true);
    // }


    // Setting for SLAM and navigation (odometry, joint states, TF)
    initOdom();

    initJointStates();

    prev_update_time = millis();

    Serial.print("Connecting to the motor drivers...");
    delay(1000);
    if (!motor_driver.is_connected())
    {
        Serial.println("Failing to connect to the motor drivers");
        while(1);
    }
    motor_driver.set_torque(true);
    Serial.println("Success to initialize and connect the motor drivers");

    max_linear_velocity = burger_info.wheel_radius * 2 * PI * burger_info.model_motor_rpm / 60;
    min_linear_velocity = -max_linear_velocity;
    max_angular_velocity = max_linear_velocity / burger_info.turning_radius;
    min_angular_velocity = -max_angular_velocity;

    // Init IMU
    sensors.initIMU();
    sensors.calibrationGyro();

    // Initialize WiFi module and connect to network
    if (wifiInit() < 0)
    {
        Serial.println("WiFi shield not present");
        while (true);
    }
    Serial.println("WiFi Connected!");

    // Initialize Zenoh Session and other parameters
    z_owned_config_t config = z_config_default();
    zp_config_insert(z_config_loan(&config), Z_CONFIG_MODE_KEY, z_string_make(MODE));
    if (strcmp(PEER, "") != 0) {
        zp_config_insert(z_config_loan(&config), Z_CONFIG_PEER_KEY, z_string_make(PEER));
    }

    // Open Zenoh session
    Serial.print("Opening Zenoh Session...");
    s = z_open(z_config_move(&config));
    if (!z_session_check(&s)) {
        Serial.print("Unable to open session!\n");
        while(1);
    }
    Serial.println("OK");

    delay(2000);
    unsigned long rid = 0;

    // Declaring Publications
    z_owned_publisher_t pub_sensor_state = z_declare_publisher(z_session_loan(&s), z_keyexpr(SENSOR_STATE), NULL);
    if (!z_publisher_check(&pub_sensor_state)) {
        while(1);
    }

    delay(100);

    z_owned_publisher_t pub_firmware_version = z_declare_publisher(z_session_loan(&s), z_keyexpr(FIRMWARE_VERSION), NULL);
    if (!z_publisher_check(&pub_firmware_version)) {
        while(1);
    }

    delay(100);

    z_owned_publisher_t pub_imu = z_declare_publisher(z_session_loan(&s), z_keyexpr(IMU), NULL);
    if (!z_publisher_check(&pub_imu)) {
        while(1);
    }

    delay(100);

    z_owned_publisher_t pub_cmd_vel_rc100 = z_declare_publisher(z_session_loan(&s), z_keyexpr(CMD_VEL_RC100), NULL);
    if (!z_publisher_check(&pub_cmd_vel_rc100)) {
        while(1);
    }

    delay(100);

    z_owned_publisher_t pub_odom = z_declare_publisher(z_session_loan(&s), z_keyexpr(ODOM), NULL);
    if (!z_publisher_check(&pub_odom)) {
        while(1);
    }

    delay(100);

    z_owned_publisher_t pub_joint_states = z_declare_publisher(z_session_loan(&s), z_keyexpr(JOINT_STATES), NULL);
    if (!z_publisher_check(&pub_joint_states)) {
        while(1);
    }

    delay(100);

    z_owned_publisher_t pub_battery_state = z_declare_publisher(z_session_loan(&s), z_keyexpr(BATTERY_STATE), NULL);
    if (!z_publisher_check(&pub_battery_state)) {
        while(1);
    }

    delay(100);

    z_owned_publisher_t pub_magnetic_field = z_declare_publisher(z_session_loan(&s), z_keyexpr(MAGNETIC_FIELD), NULL);
    if (!z_publisher_check(&pub_magnetic_field)) {
        while(1);
    }

    delay(100);

    z_owned_publisher_t pub_broadcast_tf = z_declare_publisher(z_session_loan(&s), z_keyexpr(BROADCAST_TF), NULL);
    if (!z_publisher_check(&pub_broadcast_tf)) {
        while(1);
    }

    delay(100);

    // Declaring subscriptions
    z_owned_closure_sample_t callback_cmd_vel = z_closure_sample(commandVelocityCallback, NULL, NULL);
    z_owned_subscriber_t sub_cmd_vel = z_declare_subscriber(z_session_loan(&s), z_keyexpr(CMD_VEL), z_closure_sample_move(&callback_cmd_vel), NULL);
    if (!z_subscriber_check(&sub_cmd_vel)) {
        while(1);
    }

    delay(100);

    z_owned_closure_sample_t callback_sound = z_closure_sample(soundCallback, NULL, NULL);
    z_owned_subscriber_t sub_sound = z_declare_subscriber(z_session_loan(&s), z_keyexpr(SOUND), z_closure_sample_move(&callback_sound), NULL);
    if (!z_subscriber_check(&sub_sound)) {
        while(1);
    }

    delay(100);

    z_owned_closure_sample_t callback_motor_power = z_closure_sample(motorPowerCallback, NULL, NULL);
    z_owned_subscriber_t sub_motor_power = z_declare_subscriber(z_session_loan(&s), z_keyexpr(MOTOR_POWER), z_closure_sample_move(&callback_motor_power), NULL);
    if (!z_subscriber_check(&sub_motor_power)) {
        while(1);
    }

    delay(100);

    z_owned_closure_sample_t callback_reset = z_closure_sample(resetCallback, NULL, NULL);
    z_owned_subscriber_t sub_reset = z_declare_subscriber(z_session_loan(&s), z_keyexpr(RESET), z_closure_sample_move(&callback_reset), NULL);
    if (!z_subscriber_check(&sub_reset)) {
        while(1);
    }

    delay(100);

    pinMode(BDPIN_LED_USER_1, OUTPUT);
    pinMode(BDPIN_LED_USER_2, OUTPUT);
    pinMode(BDPIN_LED_USER_3, OUTPUT);
    pinMode(BDPIN_LED_USER_4, OUTPUT);

    initTimes();

    // Set led on to indicate that the initialization is complete.
    digitalWrite(BDPIN_LED_USER_1, false);

    // semaphore
    handle = xSemaphoreCreateMutex();

    // define thread
    osThreadDef(THREAD_NAME_LOOP, run_loop, osPriorityNormal, 1, 2560);
    osThreadDef(THREAD_NAME_READ, run_read_task, osPriorityNormal, 1, 5120);
    osThreadDef(THREAD_NAME_LEASE, run_lease_task, osPriorityNormal, 1, 1280);

    // create thread
    thread_id_loop = osThreadCreate(osThread(THREAD_NAME_LOOP), NULL);
    thread_id_read  = osThreadCreate(osThread(THREAD_NAME_READ), NULL);
    thread_id_lease  = osThreadCreate(osThread(THREAD_NAME_LEASE), NULL);


    sensors.makeMelody(1);
    // start kernel
    osKernelStart();
}

void loop()
{

    // Serial.println(".");
    digitalWrite(BDPIN_LED_USER_3, !digitalRead(BDPIN_LED_USER_3));

    sensors.onMelody();

    // Update Voltage
    // diagnosis.updateVoltageCheck(true);

    // Update the IMU unit
    sensors.updateIMU();

    uint32_t t = millis();


    if ((t-tTime[0]) >= (1000 / CONTROL_MOTOR_SPEED_FREQUENCY))
    {
        updateGoalVelocity();

        if ((t-tTime[6]) > CONTROL_MOTOR_TIMEOUT)
        {
            motor_driver.control_motors(burger_info.wheel_separation, zero_velocity[VelocityType::LINEAR], zero_velocity[VelocityType::ANGULAR]);
        }
        else
        {
            motor_driver.control_motors(burger_info.wheel_separation, goal_velocity[VelocityType::LINEAR], goal_velocity[VelocityType::ANGULAR]);
        }

        tTime[0] = t;
    }
    Serial.println("OK");

    if ((t-tTime[2]) >= (1000 / DRIVE_INFORMATION_PUBLISH_FREQUENCY))
    {
        // Sends sensors state
        publishSensorStateMsg(z_publisher_loan(&pub_sensor_state));

         // Sends battery state
        publishBatteryStateMsg(z_publisher_loan(&pub_battery_state));

        // Sends drive informations
        publishDriveInformation(z_publisher_loan(&pub_odom), z_publisher_loan(&pub_broadcast_tf), z_publisher_loan(&pub_joint_states));
        tTime[2] = t;
    }

    if ((t-tTime[3]) >= (1000 / IMU_PUBLISH_FREQUENCY))
    {
        // Sends the IMU status
        publishImuMsg(z_publisher_loan(&pub_imu));

        // Sends the Magnetic Status
        publishMagMsg(z_publisher_loan(&pub_magnetic_field));
        tTime[3] = t;

    }

    if ((t-tTime[4]) >= (1000 / VERSION_INFORMATION_PUBLISH_FREQUENCY))
    {
        // Sends version info
        publishVersionInfoMsg(z_publisher_loan(&pub_firmware_version));
        tTime[4] = t;
        digitalWrite(BDPIN_LED_USER_4, !digitalRead(BDPIN_LED_USER_4));
    }
}

/*******************************************************************************
* Callback function for reset msg
*******************************************************************************/
void resetCallback(const z_sample_t *sample, void *arg)
{
  reset_msg.deserialize((unsigned char*)sample->payload.start);

  Serial.println("Start Calibration of Gyro");

  sensors.calibrationGyro();

  Serial.println("Calibration End");

  initOdom();

  Serial.println("Reset Odometry");
}

/*******************************************************************************
* Callback function for motor_power msg
*******************************************************************************/
void motorPowerCallback(const z_sample_t *sample, void *arg)
{
  motor_power_msg.deserialize((unsigned char*)sample->payload.start);

  motor_driver.set_torque(motor_power_msg.data);
}

/*******************************************************************************
* Callback function for sound msg
*******************************************************************************/
void soundCallback(const z_sample_t *sample, void *arg)
{
  sound_msg.deserialize((unsigned char*)sample->payload.start);
  sensors.makeMelody(sound_msg.value);
}

/*******************************************************************************
* Callback function for cmd_vel msg
*******************************************************************************/
void commandVelocityCallback(const z_sample_t *sample, void *arg)
{
    (void)(arg); // Unused argument
    led_4_status = !led_4_status;

    cmd_vel_msg.deserialize((unsigned char*)sample->payload.start);

    goal_velocity_from_cmd[VelocityType::LINEAR] = constrain((float)(cmd_vel_msg.linear.x), MIN_LINEAR_VELOCITY, MAX_LINEAR_VELOCITY);
    goal_velocity_from_cmd[VelocityType::ANGULAR] = constrain((float)(cmd_vel_msg.angular.z), MIN_ANGULAR_VELOCITY, MAX_ANGULAR_VELOCITY);
    tTime[6] = millis();
}

/*******************************************************************************
* Publish msgs (IMU data: angular velocity, linear acceleration, orientation)
*******************************************************************************/
void publishImuMsg(z_publisher_t *pub)
{
    float *angular_velocity = sensors.getImuAngularVelocity();
    float *linear_velocity = sensors.getImuLinearAcc();
    float *orientation = sensors.getOrientation();

    imu_msg.angular_velocity.x = angular_velocity[0];
    imu_msg.angular_velocity.y = angular_velocity[1];
    imu_msg.angular_velocity.z = angular_velocity[2];

    imu_msg.angular_velocity_covariance[0] = 0.02;
    imu_msg.angular_velocity_covariance[1] = 0;
    imu_msg.angular_velocity_covariance[2] = 0;
    imu_msg.angular_velocity_covariance[3] = 0;
    imu_msg.angular_velocity_covariance[4] = 0.02;
    imu_msg.angular_velocity_covariance[5] = 0;
    imu_msg.angular_velocity_covariance[6] = 0;
    imu_msg.angular_velocity_covariance[7] = 0;
    imu_msg.angular_velocity_covariance[8] = 0.02;

    imu_msg.linear_acceleration.x = linear_velocity[0];
    imu_msg.linear_acceleration.y = linear_velocity[1];
    imu_msg.linear_acceleration.z = linear_velocity[2];

    imu_msg.linear_acceleration_covariance[0] = 0.04;
    imu_msg.linear_acceleration_covariance[1] = 0;
    imu_msg.linear_acceleration_covariance[2] = 0;
    imu_msg.linear_acceleration_covariance[3] = 0;
    imu_msg.linear_acceleration_covariance[4] = 0.04;
    imu_msg.linear_acceleration_covariance[5] = 0;
    imu_msg.linear_acceleration_covariance[6] = 0;
    imu_msg.linear_acceleration_covariance[7] = 0;
    imu_msg.linear_acceleration_covariance[8] = 0.04;

    imu_msg.orientation.w = orientation[0];
    imu_msg.orientation.x = orientation[1];
    imu_msg.orientation.y = orientation[2];
    imu_msg.orientation.z = orientation[3];

    imu_msg.orientation_covariance[0] = 0.0025;
    imu_msg.orientation_covariance[1] = 0;
    imu_msg.orientation_covariance[2] = 0;
    imu_msg.orientation_covariance[3] = 0;
    imu_msg.orientation_covariance[4] = 0.0025;
    imu_msg.orientation_covariance[5] = 0;
    imu_msg.orientation_covariance[6] = 0;
    imu_msg.orientation_covariance[7] = 0;
    imu_msg.orientation_covariance[8] = 0.0025;


    imu_msg.header.stamp = fake_time;
    imu_msg.header.frame_id = imu_frame_id;

    int size = imu_msg.serialize(buf);

    z_publisher_put(pub, (const uint8_t *)buf, size, NULL);
}

/*******************************************************************************
* Publish msgs (Magnetic data)
*******************************************************************************/
void publishMagMsg(z_publisher_t *pub)
{

    float *magnetic = sensors.getImuMagnetic();

    mag_msg.magnetic_field.x = magnetic[0];
    mag_msg.magnetic_field.y = magnetic[1];
    mag_msg.magnetic_field.z = magnetic[2];

    mag_msg.magnetic_field_covariance[0] = 0.0048;
    mag_msg.magnetic_field_covariance[1] = 0;
    mag_msg.magnetic_field_covariance[2] = 0;
    mag_msg.magnetic_field_covariance[3] = 0;
    mag_msg.magnetic_field_covariance[4] = 0.0048;
    mag_msg.magnetic_field_covariance[5] = 0;
    mag_msg.magnetic_field_covariance[6] = 0;
    mag_msg.magnetic_field_covariance[7] = 0;
    mag_msg.magnetic_field_covariance[8] = 0.0048;

    mag_msg.header.stamp    = fake_time;
    mag_msg.header.frame_id = mag_frame_id;

    int size = mag_msg.serialize(buf);
    z_publisher_put(pub, (const uint8_t *)buf, size, NULL);
}

/*******************************************************************************
* Publish msgs (sensor_state: bumpers, cliffs, buttons, encoders, battery)
*******************************************************************************/
void publishSensorStateMsg(z_publisher_t *pub)
{
    bool dxl_comm_result = false;

    sensor_state_msg.header.stamp = fake_time;
    sensor_state_msg.battery = sensors.checkVoltage();

    dxl_comm_result = motor_driver.read_present_position(sensor_state_msg.left_encoder, sensor_state_msg.right_encoder);

    if (dxl_comm_result == true)
        updateMotorInfo(sensor_state_msg.left_encoder, sensor_state_msg.right_encoder);
    else
        return;

    sensor_state_msg.bumper = sensors.checkPushBumper();

    sensor_state_msg.cliff = sensors.getIRsensorData();

    // TODO
    // sensor_state_msg.sonar = sensors.getSonarData();

    sensor_state_msg.illumination = sensors.getIlluminationData();

    sensor_state_msg.button = sensors.checkPushButton();

    sensor_state_msg.torque = motor_driver.get_torque();


    int size = sensor_state_msg.serialize(buf);
    z_publisher_put(pub, (const uint8_t *)buf, size, NULL);
}

/*******************************************************************************
* Publish msgs (version info)
*******************************************************************************/
void publishVersionInfoMsg(z_publisher_t *pub)
{
    version_info_msg.hardware = "0.0.0";
    version_info_msg.software = "0.0.0";
    version_info_msg.firmware = FIRMWARE_VER;

    int size = version_info_msg.serialize(buf);
    z_publisher_put(pub, (const uint8_t *)buf, size, NULL);
}

/*******************************************************************************
* Publish msgs (battery_state)
*******************************************************************************/
void publishBatteryStateMsg(z_publisher_t *pub)
{
    battery_state_msg.header.stamp = fake_time;
    battery_state_msg.design_capacity = 1.8f; //Ah
    battery_state_msg.voltage = sensors.checkVoltage();
    battery_state_msg.percentage = (float)(battery_state_msg.voltage / 11.1f);

    if (battery_state == 0)
        battery_state_msg.present = false;
    else
        battery_state_msg.present = true;

    int size = battery_state_msg.serialize(buf);
    z_publisher_put(pub, (const uint8_t *)buf, size, NULL);
}

/*******************************************************************************
* Publish msgs (odometry, joint states, tf)
*******************************************************************************/
void publishDriveInformation(z_publisher_t *pub_odom, z_publisher_t *pub_tf, z_publisher_t *pub_js)
{
    unsigned long time_now = millis();
    unsigned long step_time = time_now - prev_update_time;

    prev_update_time = time_now;
    ros::Time stamp_now = fake_time;

    // calculate odometry
    calcOdometry((double)(step_time * 0.001));

    // odometry
    updateOdometry();
    odom_msg.header.stamp = stamp_now;

    int size = odom_msg.serialize(buf);
    z_publisher_put(pub_odom, (const uint8_t *)buf, size, NULL);

    // odometry tf
    updateTF(odom_tf);
    odom_tf.header.stamp = stamp_now;
    sendTransform(pub_tf);

    // joint states
    updateJointStates();
    joint_states_msg.header.stamp = stamp_now;

    size = joint_states_msg.serialize(buf);
    z_publisher_put(pub_js, (const uint8_t *)buf, size, NULL);
}

/*******************************************************************************
* Broadcast tf
*******************************************************************************/
void sendTransform(z_publisher_t *pub)
{
    tf::tfMessage internal_msg;

    internal_msg.transforms_length = 1;
    internal_msg.transforms = &odom_tf;

    int size = internal_msg.serialize(buf);
    z_publisher_put(pub, (const uint8_t *)buf, size, NULL);
}

/*******************************************************************************
* Calculate the odometry
*******************************************************************************/
bool calcOdometry(double diff_time)
{
    float* orientation;
    double wheel_l, wheel_r;      // rotation value of wheel [rad]
    double delta_s, theta, delta_theta;
    static double last_theta = 0.0;
    double v, w;                  // v = translational velocity [m/s], w = rotational velocity [rad/s]
    double step_time;

    wheel_l = wheel_r = 0.0;
    delta_s = delta_theta = theta = 0.0;
    v = w = 0.0;
    step_time = 0.0;

    step_time = diff_time;

    if (step_time == 0)
        return false;

    wheel_l = TICK2RAD * (double)last_diff_tick[LEFT];
    wheel_r = TICK2RAD * (double)last_diff_tick[RIGHT];

    if (isnan(wheel_l))
        wheel_l = 0.0;

    if (isnan(wheel_r))
        wheel_r = 0.0;

    delta_s     = burger_info.wheel_radius * (wheel_r + wheel_l) / 2.0;
    // theta = WHEEL_RADIUS * (wheel_r - wheel_l) / WHEEL_SEPARATION;
    orientation = sensors.getOrientation();
    theta       = atan2f(orientation[1]*orientation[2] + orientation[0]*orientation[3],
                    0.5f - orientation[2]*orientation[2] - orientation[3]*orientation[3]);

    delta_theta = theta - last_theta;

    // compute odometric pose
    odom_pose[0] += delta_s * cos(odom_pose[2] + (delta_theta / 2.0));
    odom_pose[1] += delta_s * sin(odom_pose[2] + (delta_theta / 2.0));
    odom_pose[2] += delta_theta;

    // compute odometric instantaneouse velocity

    v = delta_s / step_time;
    w = delta_theta / step_time;

    odom_vel[0] = v;
    odom_vel[1] = 0.0;
    odom_vel[2] = w;

    last_velocity[LEFT]  = wheel_l / step_time;
    last_velocity[RIGHT] = wheel_r / step_time;
    last_theta = theta;

    return true;
}

/*******************************************************************************
* Update the odometry
*******************************************************************************/
void updateOdometry(void)
{
    odom_msg.header.frame_id = odom_header_frame_id;
    odom_msg.child_frame_id  = odom_child_frame_id;

    odom_msg.pose.pose.position.x = odom_pose[0];
    odom_msg.pose.pose.position.y = odom_pose[1];
    odom_msg.pose.pose.position.z = 0;
    odom_msg.pose.pose.orientation = tf::createQuaternionFromYaw(odom_pose[2]);

    odom_msg.twist.twist.linear.x  = odom_vel[0];
    odom_msg.twist.twist.angular.z = odom_vel[2];
}

/*******************************************************************************
* Initialization odometry data
*******************************************************************************/
void initOdom(void)
{
  init_encoder = true;

  for (int index = 0; index < 3; index++)
  {
    odom_pose[index] = 0.0;
    odom_vel[index]  = 0.0;
  }

  odom_msg.pose.pose.position.x = 0.0;
  odom_msg.pose.pose.position.y = 0.0;
  odom_msg.pose.pose.position.z = 0.0;

  odom_msg.pose.pose.orientation.x = 0.0;
  odom_msg.pose.pose.orientation.y = 0.0;
  odom_msg.pose.pose.orientation.z = 0.0;
  odom_msg.pose.pose.orientation.w = 0.0;

  odom_msg.twist.twist.linear.x  = 0.0;
  odom_msg.twist.twist.angular.z = 0.0;
}

/*******************************************************************************
* CalcUpdateulate the TF
*******************************************************************************/
void updateTF(geometry_msgs::TransformStamped& odom_tf)
{
    odom_tf.header = odom_msg.header;
    odom_tf.child_frame_id = odom_msg.child_frame_id;
    odom_tf.transform.translation.x = odom_msg.pose.pose.position.x;
    odom_tf.transform.translation.y = odom_msg.pose.pose.position.y;
    odom_tf.transform.translation.z = odom_msg.pose.pose.position.z;
    odom_tf.transform.rotation      = odom_msg.pose.pose.orientation;
}

/*******************************************************************************
* Update the joint states
*******************************************************************************/
void updateJointStates(void)
{
    static float joint_states_pos[WHEEL_NUM] = {0.0, 0.0};
    static float joint_states_vel[WHEEL_NUM] = {0.0, 0.0};
    //static float joint_states_eff[WHEEL_NUM] = {0.0, 0.0};

    joint_states_pos[LEFT]  = last_rad[LEFT];
    joint_states_pos[RIGHT] = last_rad[RIGHT];

    joint_states_vel[LEFT]  = last_velocity[LEFT];
    joint_states_vel[RIGHT] = last_velocity[RIGHT];

    joint_states_msg.position = joint_states_pos;
    joint_states_msg.velocity = joint_states_vel;
}

/*******************************************************************************
* Initialization joint states data
*******************************************************************************/
void initJointStates(void)
{
  static char *joint_states_name[] = {(char*)"wheel_left_joint", (char*)"wheel_right_joint"};

  joint_states_msg.header.frame_id = joint_state_header_frame_id;
  joint_states_msg.name            = joint_states_name;

  joint_states_msg.name_length     = WHEEL_NUM;
  joint_states_msg.position_length = WHEEL_NUM;
  joint_states_msg.velocity_length = WHEEL_NUM;
  joint_states_msg.effort_length   = WHEEL_NUM;
}

/*******************************************************************************
* Update motor information
*******************************************************************************/
void updateMotorInfo(int32_t left_tick, int32_t right_tick)
{
  int32_t current_tick = 0;
  static int32_t last_tick[WHEEL_NUM] = {0, 0};

  if (init_encoder)
  {
    for (int index = 0; index < WHEEL_NUM; index++)
    {
      last_diff_tick[index] = 0;
      last_tick[index]      = 0;
      last_rad[index]       = 0.0;

      last_velocity[index]  = 0.0;
    }

    last_tick[LEFT] = left_tick;
    last_tick[RIGHT] = right_tick;

    init_encoder = false;
    return;
  }

  current_tick = left_tick;

  last_diff_tick[LEFT] = current_tick - last_tick[LEFT];
  last_tick[LEFT]      = current_tick;
  last_rad[LEFT]       += TICK2RAD * (double)last_diff_tick[LEFT];

  current_tick = right_tick;

  last_diff_tick[RIGHT] = current_tick - last_tick[RIGHT];
  last_tick[RIGHT]      = current_tick;
  last_rad[RIGHT]       += TICK2RAD * (double)last_diff_tick[RIGHT];
}


/*******************************************************************************
* Update Goal Velocity
*******************************************************************************/
void updateGoalVelocity(void)
{
  goal_velocity[VelocityType::LINEAR]  = goal_velocity_from_cmd[VelocityType::LINEAR];
  goal_velocity[VelocityType::ANGULAR] =  goal_velocity_from_cmd[VelocityType::ANGULAR];

  sensors.setLedPattern(goal_velocity[VelocityType::LINEAR], goal_velocity[VelocityType::ANGULAR]);
}

/*******************************************************************************
* init times
*******************************************************************************/
void initTimes(void)
{
    uint32_t now = millis();
    for(int i = 0; i < 10; i++)
        tTime[i] = now;
}

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

#include <Arduino.h>
#include <WiFi.h>

#include <Wire.h>
#include <MPU6050_tockn.h>

// For ROS types
#include <geometry_msgs/Twist.h>


extern "C" {
    #include "zenoh-pico.h"
}

// WiFi-specific parameters

#define SSID "NAME"
#define PASS "PASSWORD"

// Zenoh-specific parameters
#define MODE "client"
#define PEER "tcp/192.168.86.57:7447"

#define URI "rt/cmd_vel"

// Measurement specific parameters
#define X_SCALING_FACTOR 100.0
#define X_MAX_VALUE 0.20
#define X_MIN_VALUE -0.20
#define X_ZERO_VALUE 0.10
#define Y_SCALING_FACTOR 10.0
#define Y_MAX_VALUE 2.50
#define Y_MIN_VALUE -2.50
#define Y_ZERO_VALUE 0.5

#define CONTROL_MOTOR_SPEED_FREQUENCY 30 //hz

/* ---------- Print Functions ----------- */
void printVector(geometry_msgs::Vector3 *v)
{
    Serial.print("X: ");
    Serial.print(v->x);
    Serial.print(", Y: ");
    Serial.print(v->y);
    Serial.print(", Z: ");
    Serial.print(v->z);
}

void printTwist(geometry_msgs::Twist *t)
{
    Serial.print("Linear ");
    printVector(&t->linear);
    Serial.println("");

    Serial.print("Angular ");
    printVector(&t->angular);
    Serial.println("");
}

/* -------------------------------------- */

MPU6050 mpu(Wire);

z_owned_session_t s;
z_owned_publisher_t pub_sensor_state;

double offset_x = 0.0;
double offset_y = 0.0;

z_owned_publisher_t pub;

void setup(void)
{
    // Initialize Serial for debug
    Serial.begin(115200);
    // while (!Serial)
    //     delay(10);


    // Set WiFi in STA mode and trigger attachment
    WiFi.mode(WIFI_STA);
    WiFi.begin(SSID, PASS);
    while (WiFi.status() != WL_CONNECTED) {
        delay(1000);
    }
    Serial.println("Connected to WiFi!");

    // Initialize and calibrate0 MPU6050
    Serial.print("Detecting MPU6050 sensor...");
    Wire.begin();
    mpu.begin();
    mpu.calcGyroOffsets(true);

    Serial.println("MPU6050 Found!");

    // Initialize Zenoh Session and other parameters
    z_owned_config_t config = zp_config_default();
    zp_config_insert(z_config_loan(&config), Z_CONFIG_MODE_KEY, z_string_make(MODE));
    if (strcmp(PEER, "") != 0) {
        zp_config_insert(z_config_loan(&config), Z_CONFIG_PEER_KEY, z_string_make(PEER));
    }

    s = z_open(z_config_move(&config));
    if (!z_session_check(&s)) {
        Serial.print("Unable to open session!\n");
        while(1);
    }

    zp_start_read_task(z_session_loan(&s));
    zp_start_lease_task(z_session_loan(&s));

    pub_sensor_state = z_declare_publisher(z_session_loan(&s), z_keyexpr(URI), NULL);
    if (!z_publisher_check(&pub_sensor_state)) {
        while(1);
    }
    Serial.println("Zenoh Publisher setup finished!");

    delay(5000);

    mpu.update();
    offset_x = mpu.getAccAngleX();
    offset_y = mpu.getAccAngleY();
    Serial.println("OK");

    delay(300);

    // Initialize Zenoh Session and other parameters
    z_owned_config_t config = zp_config_default();
    zp_config_insert(z_config_loan(&config), Z_CONFIG_MODE_KEY, z_string_make(MODE));
    if (strcmp(PEER, "") != 0) {
        zp_config_insert(z_config_loan(&config), Z_CONFIG_PEER_KEY, z_string_make(PEER));
    }

    // Open Zenoh session
    Serial.print("Opening Zenoh Session...");
    z_owned_session_t s = z_open(z_config_move(&config));
    if (!z_session_check(&s)) {
        Serial.println("Unable to open session!\n");
        while(1);
    }
    Serial.println("OK");

    // Start the receive and the session lease loop for zenoh-pico
    zp_start_read_task(z_session_loan(&s));
    zp_start_lease_task(z_session_loan(&s));

    // Declare Zenoh publisher
    Serial.print("Declaring publisher for ");
    Serial.print(KEYEXPR);
    Serial.println("...");
    pub = z_declare_publisher(z_session_loan(&s), z_keyexpr(KEYEXPR), NULL);
    if (!z_publisher_check(&pub)) {
        Serial.println("Unable to declare publisher for key expression!\n");
        while(1);
    }
    Serial.println("OK");
    Serial.println("Zenoh setup finished!");
}

void loop()
{
    delay(1000 / CONTROL_MOTOR_SPEED_FREQUENCY);

    mpu.update();

    double linear_x = (mpu.getAccAngleX() - offset_x) / X_SCALING_FACTOR;
    double linear_y = (mpu.getAccAngleY() - offset_y) / Y_SCALING_FACTOR;
    linear_x = min(max(linear_x, X_MIN_VALUE), X_MAX_VALUE);
    if (linear_x < X_ZERO_VALUE && linear_x > -X_ZERO_VALUE)
        linear_x = 0;
    linear_y = min(max(linear_y, Y_MIN_VALUE), Y_MAX_VALUE);
    if (linear_y < Y_ZERO_VALUE && linear_y > -Y_ZERO_VALUE)
        linear_y = 0;

    // Reusing micro-ROS(1) types
    geometry_msgs::Twist cmd_vel_msg;

    cmd_vel_msg.linear.x = linear_x;
    cmd_vel_msg.linear.y = 0.0;
    cmd_vel_msg.linear.z = 0.0;
    cmd_vel_msg.angular.x = 0.0;
    cmd_vel_msg.angular.y = 0.0;
    cmd_vel_msg.angular.z = linear_y;

    printTwist(&cmd_vel_msg);
    Serial.println("");

    uint8_t twist_serialized_size = 4 + sizeof(double) * 6;
    unsigned char buf[twist_serialized_size];
    cmd_vel_msg.serialize(buf);
    z_publisher_put(z_publisher_loan(&pub_sensor_state), (const uint8_t *)buf, twist_serialized_size, NULL);
}

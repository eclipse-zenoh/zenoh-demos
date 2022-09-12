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

extern "C" {
    #include "zenoh-pico.h"
}

// WiFi-specific parameters
#define SSID "SSID"
#define PASS "PASS"

// Zenoh-specific parameters
#define MODE "client"
#define PEER ""

#define TURTLESIM 1
#if TURTLESIM == 1
    #define KEYEXPR "rt/turtle1/cmd_vel"
#else
    #define KEYEXPR "rt/cmd_vel"
#endif

// Measurement specific parameters
#if TURTLESIM == 1
    #define X_SCALING_FACTOR 10.0
    #define X_MAX_VALUE 2.00
    #define X_MIN_VALUE -2.00
    #define X_ZERO_VALUE 0.5
#else
    #define X_SCALING_FACTOR 100.0
    #define X_MAX_VALUE 0.20
    #define X_MIN_VALUE -0.20
    #define X_ZERO_VALUE 0.10
#endif

#define Y_SCALING_FACTOR 10.0
#define Y_MAX_VALUE 2.80
#define Y_MIN_VALUE -2.80
#define Y_ZERO_VALUE 0.5

/* --------------- Structs -------------- */
struct Vector3
{
    double x;
    double y;
    double z;
};

struct Twist
{
    Vector3 linear;
    Vector3 angular;
};

/* -------- Serialize Functions --------- */
char *serialize_float_as_f64_little_endian(double val, char *buf)
{
    long long *c_val = (long long*)&val;
    for (int i = 0; i < sizeof(double); ++i, ++buf) {
       *buf = 0xFF & (*c_val >> (i * 8));
    }

    return buf;
}

char *serialize_vector3(Vector3 *v, char *buf)
{
    buf = serialize_float_as_f64_little_endian(v->x, buf);
    buf = serialize_float_as_f64_little_endian(v->y, buf);
    buf = serialize_float_as_f64_little_endian(v->z, buf);

    return buf;
}

void serialize_twist(Twist *t, char *buf)
{
    // Serialize Twist header for little endian
    *(buf++) = 0x00;
    *(buf++) = 0x01;
    *(buf++) = 0x00;
    *(buf++) = 0x00;
    buf = serialize_vector3(&t->linear, buf);
    buf = serialize_vector3(&t->angular, buf);
}

/* ---------- Print Functions ----------- */
void printVector(struct Vector3 *v)
{
    Serial.print("X: ");
    Serial.print(v->x);
    Serial.print(", Y: ");
    Serial.print(v->y);
    Serial.print(", Z: ");
    Serial.print(v->z);
}

void printTwist(struct Twist *t)
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
double offset_x = 0.0;
double offset_y = 0.0;

z_owned_publisher_t pub;

void setup(void)
{
    // Initialize Serial for debug
    Serial.begin(115200);
    while (!Serial) {
        delay(1000);
    }

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
    Serial.println("OK");

    Serial.print("Calibrating MPU6050 sensor...");
    mpu.update();
    offset_x = mpu.getAccAngleX();
    offset_y = mpu.getAccAngleY();
    Serial.println("OK");

    delay(300);

    // Initialize Zenoh Session and other parameters
    z_owned_config_t config = z_config_default();
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
    zp_start_read_task(z_session_loan(&s), NULL);
    zp_start_lease_task(z_session_loan(&s), NULL);

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

    delay(300);
}

void loop()
{
    delay(100);
    mpu.update();

    // Read MPU6050 sensor values and scale them
    double linear_x = (mpu.getAccAngleX() - offset_x) / X_SCALING_FACTOR;
    double linear_y = (mpu.getAccAngleY() - offset_y) / Y_SCALING_FACTOR;
    linear_x = min(max(linear_x, X_MIN_VALUE), X_MAX_VALUE);
    if (linear_x < X_ZERO_VALUE && linear_x > -X_ZERO_VALUE) {
        linear_x = 0;
    }
    linear_y = min(max(linear_y, Y_MIN_VALUE), Y_MAX_VALUE);
    if (linear_y < Y_ZERO_VALUE && linear_y > -Y_ZERO_VALUE) {
        linear_y = 0;
    }

    // Create ROS twist message
    Twist measure;
    measure.linear.x = linear_x * -1;
    measure.linear.y = 0.0;
    measure.linear.z = 0.0;
    measure.angular.x = 0.0;
    measure.angular.y = 0.0;
    measure.angular.z = linear_y;

    printTwist(&measure);
    Serial.println("");

    uint8_t twist_serialized_size = 4 + sizeof(double) * 6;
    char buf[twist_serialized_size];
    serialize_twist(&measure, buf);
    if (z_publisher_put(z_publisher_loan(&pub), (const uint8_t *)buf, twist_serialized_size, NULL) < 0) {
        Serial.println("Error while publishing data");
    }
}

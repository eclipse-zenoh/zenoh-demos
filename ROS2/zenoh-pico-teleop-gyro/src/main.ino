/*
 * Copyright (c) 2017, 2021 ADLINK Technology Inc.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Eclipse Public License 2.0 which is available at
 * http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
 * which is available at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
 *
 * Contributors:
 *   ADLINK zenoh team, <zenoh@adlink-labs.tech>
 */

#include <Arduino.h>

#include <WiFi.h>

#include <Wire.h>
#include <MPU6050_tockn.h>

extern "C" {
    #include "zenoh-pico.h"
}

// WiFi-specific parameters
#define SSID "SSID"
#define PASS "PASSWORD"

// Zenoh-specific parameters
#define MODE "client"
#define PEER ""
#define URI "/rt/turtle1/cmd_vel"

// Measurement specific parameters
#define SCALING_FACTOR 10
#define MAX_VALUE 3.0
#define MIN_VALUE -3.0


/* --------------- Structs -------------- */
struct Vector3 {
    double x;
    double y;
    double z;
};

struct Twist {
    Vector3 linear;
    Vector3 angular;
};

/* -------- Serialize Functions --------- */
char *serialize_float_as_f64_little_endian(double val, char *buf)
{
    long long *c_val = (long long*)&val;
    for (int i = 0; i < sizeof(double); ++i, ++buf)
       *buf = 0xFF & (*c_val >> (i * 8));

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
zn_session_t *s = NULL;
zn_reskey_t *reskey = NULL;
double offset_x = 0.0;
double offset_y = 0.0;

void setup(void) {
    // Initialize Serial for debug
    Serial.begin(115200);
    while (!Serial)
        delay(10);

    // Set WiFi in STA mode and trigger attachment
    WiFi.mode(WIFI_STA);
    WiFi.begin(SSID, PASS);
    while (WiFi.status() != WL_CONNECTED)
        delay(1000);
    Serial.println("Connected to WiFi!");
  
    // Initialize MPU6050
    Wire.begin();
    mpu.begin();
    mpu.calcGyroOffsets(true);
    Serial.println("MPU6050 Found!");
  
    // Initialize Zenoh Session and other parameters
    zn_properties_t *config = zn_config_default();
    zn_properties_insert(config, ZN_CONFIG_MODE_KEY, z_string_make(MODE));
    if (strcmp(PEER, "") != 0)
        zn_properties_insert(config, ZN_CONFIG_PEER_KEY, z_string_make(PEER));

    s = zn_open(config);
    if (s == NULL)
    {
        return;
    }

    znp_start_read_task(s);
    znp_start_lease_task(s);

    unsigned long rid = zn_declare_resource(s, zn_rname(URI));
    reskey = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *reskey = zn_rid(rid);

    zn_publisher_t *pub = zn_declare_publisher(s, *reskey);
    if (pub == NULL) {
        return;
    }
    Serial.println("Zenoh Publisher setup finished!");

    mpu.update();
    offset_x = mpu.getAccAngleX();
    offset_y = mpu.getAccAngleY();
    delay(1000);
}

void loop() {
    delay(1000);
    mpu.update();
  
    double linear_x = (mpu.getAccAngleX() - offset_x) / SCALING_FACTOR;
    double linear_y = (mpu.getAccAngleY() - offset_y) / SCALING_FACTOR;
    linear_x = min(max(linear_x, MIN_VALUE), MAX_VALUE);
    if (linear_x < 1 && linear_x > -1)
        linear_x = 0;
    linear_y = min(max(linear_y, MIN_VALUE), MAX_VALUE);
    if (linear_y < 1 && linear_y > -1)
        linear_y = 0;

    Twist measure;
    measure.linear.x = linear_x;
    measure.linear.y = 0.0;
    measure.linear.z = 0.0;
    measure.angular.x = 0.0;
    measure.angular.y = 0.0;
    measure.angular.z = linear_y;

    printTwist(&measure);
    Serial.println("");
  
    if (s == NULL)
        return;

    if (reskey == NULL)
        return;

    uint8_t twist_serialized_size = 4 + sizeof(double) * 6;
    char buf[twist_serialized_size];
    serialize_twist(&measure, buf);
    zn_write(s, *reskey, (const uint8_t *)buf, twist_serialized_size);
}

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

// For ROS2 types
#include <geometry_msgs/msg/twist.h>
#include <rosidl_typesupport_microxrcedds_c/message_type_support.h>
#include <micro_ros_utilities/type_utilities.h>
#include <ucdr/microcdr.h>


extern "C" {
    #include "zenoh-pico.h"
}

// WiFi-specific parameters
#define SSID "SSID"
#define PASS "PASS"

// Zenoh-specific parameters
#define MODE "client"
#define PEER "tcp/192.168.86.239:7887"

#define URI "/rt/cmd_vel"

// Measurement specific parameters
#define X_SCALING_FACTOR 100.0
#define X_MAX_VALUE 0.20
#define X_MIN_VALUE -0.20
#define X_ZERO_VALUE 0.10
#define Y_SCALING_FACTOR 10.0
#define Y_MAX_VALUE 2.50
#define Y_MIN_VALUE -2.50
#define Y_ZERO_VALUE 0.5




/* ---------- Print Functions ----------- */
void printVector(geometry_msgs__msg__Vector3 *v)
{
    Serial.print("X: ");
    Serial.print(v->x);
    Serial.print(", Y: ");
    Serial.print(v->y);
    Serial.print(", Z: ");
    Serial.print(v->z);
}

void printTwist(geometry_msgs__msg__Twist *t)
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
        return;

    znp_start_read_task(s);
    znp_start_lease_task(s);

    unsigned long rid = zn_declare_resource(s, zn_rname(URI));
    reskey = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *reskey = zn_rid(rid);
    Serial.println("Zenoh Publisher setup finished!");

    mpu.update();
    offset_x = mpu.getAccAngleX();
    offset_y = mpu.getAccAngleY();
    delay(1000);
}

void loop() {
    delay(100);
    mpu.update();

    double linear_x = (mpu.getAccAngleX() - offset_x) / X_SCALING_FACTOR;
    double linear_y = (mpu.getAccAngleY() - offset_y) / Y_SCALING_FACTOR;
    linear_x = min(max(linear_x, X_MIN_VALUE), X_MAX_VALUE);
    if (linear_x < X_ZERO_VALUE && linear_x > -X_ZERO_VALUE)
        linear_x = 0;
    linear_y = min(max(linear_y, Y_MIN_VALUE), Y_MAX_VALUE);
    if (linear_y < Y_ZERO_VALUE && linear_y > -Y_ZERO_VALUE)
        linear_y = 0;

    // Reusing micro-ROS types
    geometry_msgs__msg__Twist *measure = NULL;
    const rosidl_message_type_support_t *twist_type_support = ROSIDL_GET_MSG_TYPE_SUPPORT(geometry_msgs, msg, Twist);
    const message_type_support_callbacks_t *twist_ts_callbacks = (const message_type_support_callbacks_t *) twist_type_support->data;

    if (twist_ts_callbacks == NULL) {
        Serial.println("Type support callback is NULL!!!!");
        delay(5000);
        return;
    }

    if (twist_ts_callbacks->cdr_serialize == NULL) {
        Serial.println("Type support callback is NULL!!!!");
        delay(5000);
        return;
    }

    if (twist_ts_callbacks->get_serialized_size == NULL) {
        Serial.println("Type support callback is NULL!!!!");
        delay(5000);
        return;
    }


    Serial.println("Allocating message...");
    delay(1000);

    measure = geometry_msgs__msg__Twist__create();

    if (measure == NULL ){
        Serial.println("Unable to allocate!");
        delay(5000);
        return;
    }

    Serial.println("Allocated, initializing....");
    delay(1000);

    if (!geometry_msgs__msg__Twist__init(measure)){
        Serial.println("Unable to initialize!");
        delay(5000);
        return;
    }


    measure->linear.x = linear_x;
    measure->linear.y = 0.0;
    measure->linear.z = 0.0;
    measure->angular.x = 0.0;
    measure->angular.y = 0.0;
    measure->angular.z = linear_y;

    printTwist(measure);
    Serial.println("");

    if (s == NULL || reskey == NULL)
        return;

    // buffer
    uint8_t *buf = NULL;
    ucdrBuffer mb;
    size_t buf_size = 8192;


    Serial.println("Allocating buffer");
    delay(1000);
    buf = (uint8_t*)malloc(buf_size*sizeof(uint8_t));

    Serial.println("Getting serialized size buffer");
    delay(1000);
    uint32_t twist_serialized_size = twist_ts_callbacks->get_serialized_size(measure);
    Serial.print("Twist serialized size is");
    Serial.print(twist_serialized_size);

    Serial.println("Init uCDR buffer");
    delay(1000);

    ucdr_init_buffer(&mb, buf, buf_size);

    Serial.println("Twist serializing");
    delay(1000);

    twist_ts_callbacks->cdr_serialize( measure, &mb);

    // if (!twist_ts_callbacks->cdr_serialize( measure, &mb)) {
    //     Serial.println("Unable to serialize!");
    //     delay(5000);
    //     return;
    // }

    Serial.println("Twist serialized");
    delay(1000);

    // uint8_t twist_serialized_size = 4 + sizeof(double) * 6;
    // char buf[twist_serialized_size];
    // serialize_twist(&measure, buf);
    Serial.println("Sending twist");
    // zn_write(s, *reskey, (const uint8_t *)buf, twist_serialized_size);


}

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

extern "C" {
    #include "zenoh-pico.h"
}

#include "turtlebot3_motor_driver.h"

// WiFi-specific parameters
#define SSID "ATOPlay"
#define PASS "A70L@bsR0ck5!!"

// Zenoh-specific parameters
#define MODE "client"
#define PEER ""

#define KEYEXPR_CMD_VEL "rt/cmd_vel"

typedef struct TB3ModelInfo
{
    const char* model_str;
    uint32_t model_info;
    uint16_t model_motor_rpm;
    float wheel_radius;
    float wheel_separation;
    float turning_radius;
    float robot_radius;
} TB3ModelInfo;

static const TB3ModelInfo burger_info = { "Burger",
                                          1,
                                          61,
                                          0.033,
                                          0.160,
                                          0.080,
                                          0.105 };

static float max_linear_velocity, min_linear_velocity;
static float max_angular_velocity, min_angular_velocity;
static float goal_velocity[VelocityType::TYPE_NUM_MAX] = {0.0, 0.0};

bool led_status = true;
static Turtlebot3MotorDriver motor_driver;

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
* Callback function for cmd_vel msg
*******************************************************************************/
typedef struct
{
    double x;
    double y;
    double z;
} vector3_t;

typedef struct
{
    vector3_t linear;
    vector3_t angular;
} twist_t;

char *deserialize_float_as_f64_little_endian(double *val, char *buf)
{
    char *c_val = (char*)malloc(sizeof(double));
    for (int i = 0; i < sizeof(double); i++, buf++)
        c_val[i] = *buf;
  
    double *a = (double*)c_val;
    *val = *a;
    return buf;
}

char *deserialize_vector3(vector3_t *v, char *buf)
{
    buf = deserialize_float_as_f64_little_endian(&v->x, buf);
    buf = deserialize_float_as_f64_little_endian(&v->y, buf);
    buf = deserialize_float_as_f64_little_endian(&v->z, buf);
  
    return buf;
}

char *deserialize_twist(twist_t *t, char *buf)
{
    // FIXME: check the little/big endian value
    *(buf++);
    *(buf++);
    *(buf++);
    *(buf++);
  
    buf = deserialize_vector3(&t->linear, buf);
    buf = deserialize_vector3(&t->angular, buf);
  
    return buf;
}

z_owned_session_t s;

void commandVelocityCallback(const z_sample_t *sample, void *arg)
{
    (void)(arg); // Unused argument
    led_status = !led_status;

    twist_t cmd_vel_msg;
    deserialize_twist(&cmd_vel_msg, (char*)sample->payload.start);
  
    Serial.println("commandVelocityCallback");
    Serial.print("  X:");
    Serial.println(cmd_vel_msg.linear.x);
    Serial.print("  Y:");
    Serial.println(cmd_vel_msg.angular.z);

    goal_velocity[VelocityType::LINEAR] = constrain((float)(cmd_vel_msg.linear.x), min_linear_velocity, max_linear_velocity);
    goal_velocity[VelocityType::ANGULAR] = constrain((float)(cmd_vel_msg.angular.z), min_angular_velocity, max_angular_velocity);
}

void setup()
{
    // Initialize Serial for debug
    Serial.begin(115200);
    while (!Serial) {
        delay(1000);
    }

    // Set WiFi in STA mode and trigger attachment    
    WiFi.setPins(10, digitalPinToInterrupt(7), 5, -1); // Required to remap interrupt pin with opencr
    if (WiFi.status() == WL_NO_SHIELD) {
        while(1);
    }
    WiFi.begin(SSID, PASS);
    Serial.println("Connected to WiFi!");

    Serial.print("Initialize robot leds...");
    pinMode(BDPIN_LED_USER_4, OUTPUT);
    pinMode(BDPIN_LED_USER_1, OUTPUT);
    Serial.println("OK");

    Serial.print("Initialize to the motor drivers...");
    if (!motor_driver.init()) {
        Serial.println("Failing to initialize the motor drivers");
        while(1);
    }
    Serial.println("OK");

    Serial.print("Connecting to the motor drivers...");
    delay(1000);
    if (!motor_driver.is_connected())
    {
        Serial.println("Failing to connect to the motor drivers");
        while(1);
    }
    motor_driver.set_torque(true);
    Serial.println("OK");

    max_linear_velocity = burger_info.wheel_radius * 2 * PI * burger_info.model_motor_rpm / 60;
    min_linear_velocity = -max_linear_velocity;
    max_angular_velocity = max_linear_velocity / burger_info.turning_radius;
    min_angular_velocity = -max_angular_velocity;
    digitalWrite(BDPIN_LED_USER_1, LOW);

    // Initialize Zenoh Session and other parameters
    z_owned_config_t config = zp_config_default();
    zp_config_insert(z_config_loan(&config), Z_CONFIG_MODE_KEY, z_string_make(MODE));
    if (strcmp(PEER, "") != 0) {
        zp_config_insert(z_config_loan(&config), Z_CONFIG_PEER_KEY, z_string_make(PEER));
    }

    // Open Zenoh session
    Serial.print("Opening Zenoh Session...");
    s = z_open(z_config_move(&config));
    if (!z_session_check(&s)) {
        Serial.println("Unable to open session!\n");
        while(1);
    }
    Serial.println("OK");

    z_owned_closure_sample_t callback = z_closure_sample(commandVelocityCallback, NULL, NULL);
    printf("Declaring Subscriber on '%s'...\n", KEYEXPR_CMD_VEL);
    z_owned_subscriber_t sub_cmd_vel = z_declare_subscriber(z_session_loan(&s), z_keyexpr(KEYEXPR_CMD_VEL), z_closure_sample_move(&callback), NULL);
    if (!z_subscriber_check(&sub_cmd_vel))
    {
        printf("Unable to declare subscriber.\n");
        while(1);
    }
    Serial.println("OK");
    Serial.println("Zenoh setup finished!");
}

void loop()
{
    Serial.println("Loop");

    // Blink on callback
    digitalWrite(BDPIN_LED_USER_4, led_status);

    // Read Zenoh Message
    zp_read(z_session_loan(&s));

    // Send Keep Alive
    zp_send_keep_alive(z_session_loan(&s));

    // Spin over the hardware
    motor_driver.control_motors(burger_info.wheel_separation, goal_velocity[VelocityType::LINEAR], goal_velocity[VelocityType::ANGULAR]);
}

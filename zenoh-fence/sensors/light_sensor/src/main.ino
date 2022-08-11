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

extern "C" {
    #include "zenoh-pico.h"
}

// WiFi-specific parameters
#define SSID "SSID"
#define PASS "PASS"

// Zenoh-specific parameters
#define MODE "client"
#define PEER ""

#define KEYEXPR_DISTANCE "factory1/room42/distance"
#define KEYEXPR_RED_LIGHT "factory1/room42/led/red"
#define KEYEXPR_GREEN_LIGHT "factory1/room42/led/green"

#define R_PIN 25
#define G_PIN 26
#define B_PIN 27

z_owned_publisher_t pub_red;
z_owned_publisher_t pub_green;

int distance_in_cm = 0;
void data_handler(const z_sample_t *sample, void *arg)
{
    std::string val((const char*)sample->payload.start, sample->payload.len);
    distance_in_cm = std::stoi(val, NULL, 10);

    Serial.println(distance_in_cm);
}

void setColor(int red, int green, int blue)
{
    analogWrite(R_PIN, red);
    analogWrite(G_PIN, green);
    analogWrite(B_PIN, blue);
}

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

    // Light/Led initialization
    pinMode(R_PIN, OUTPUT);
    pinMode(G_PIN, OUTPUT);
    pinMode(B_PIN, OUTPUT);

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
    Serial.print(KEYEXPR_RED_LIGHT);
    Serial.println("...");
    pub_red = z_declare_publisher(z_session_loan(&s), z_keyexpr(KEYEXPR_RED_LIGHT), NULL);
    if (!z_publisher_check(&pub_red)) {
        Serial.println("Unable to declare publisher for key expression!\n");
        while(1);
    }
    Serial.println("OK");

    delay(300);

    // Declare Zenoh publisher
    Serial.print("Declaring publisher for ");
    Serial.print(KEYEXPR_GREEN_LIGHT);
    Serial.println("...");
    pub_green = z_declare_publisher(z_session_loan(&s), z_keyexpr(KEYEXPR_GREEN_LIGHT), NULL);
    if (!z_publisher_check(&pub_green)) {
        Serial.println("Unable to declare publisher for key expression!\n");
        while(1);
    }
    Serial.println("OK");

    delay(300);

    z_owned_closure_sample_t callback = z_closure_sample(data_handler, NULL, NULL);
    printf("Declaring Subscriber on '%s'...\n", KEYEXPR_DISTANCE);
    z_owned_subscriber_t sub = z_declare_subscriber(z_session_loan(&s), z_keyexpr(KEYEXPR_DISTANCE), z_closure_sample_move(&callback), NULL);
    if (!z_subscriber_check(&sub))
    {
        printf("Unable to declare subscriber.\n");
        exit(-1);
    }
    Serial.println("OK");
    Serial.println("Zenoh setup finished!");

    delay(300);
}

void loop()
{
    delay(200);

    int is_red;
    int is_green;
    if (distance_in_cm > 35)
    {
        is_red = 0;
        is_green = 1;
        setColor(0, 255, 0);
    }
    else
    {
        is_red = 1;
        is_green = 0;
        setColor(255, 0, 0);
    }

    char buf[2];
    itoa(is_red, buf, 10);
    if (z_publisher_put(z_publisher_loan(&pub_red), (const uint8_t *)buf, sizeof(buf), NULL) < 0) {
        Serial.println("Error while publishing data");
    }

    itoa(is_green, buf, 10);
    if (z_publisher_put(z_publisher_loan(&pub_green), (const uint8_t *)buf, sizeof(buf), NULL) < 0) {
        Serial.println("Error while publishing data");
    }

RET:
    delay(100);
}

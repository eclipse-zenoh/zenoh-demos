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

#include <Ultrasonic.h>

extern "C" {
    #include "zenoh-pico.h"
}

// WiFi-specific parameters
#define SSID "SSID"
#define PASS "PASS"

// Zenoh-specific parameters
#define MODE "client"
#define PEER ""

#define KEYEXPR "factory1/room42/distance"

#define RANGER_PIN 17
#define BUZZER_PIN 16

Ultrasonic ranger(RANGER_PIN);
bool buzz_state = false;

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

    // Buzzer initialization
    pinMode(BUZZER_PIN, OUTPUT);
    digitalWrite(BUZZER_PIN, buzz_state);

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
    char buf[4];
    long distance_in_cm = ranger.MeasureInCentimeters();
    Serial.print(distance_in_cm); // [0, 350]
    Serial.println(" cm");

    // Beep / Buzzer
    buzz_state = !buzz_state;
    if (distance_in_cm > 50)
        buzz_state = false;
    digitalWrite(BUZZER_PIN, buzz_state);

    // Publish the distance
    itoa(distance_in_cm, buf, 10);
    if (z_publisher_put(z_publisher_loan(&pub), (const uint8_t *)buf, sizeof(buf), NULL) < 0) {
        Serial.println("Error while publishing data");
    }

RET:
    int sleep = distance_in_cm * 10;
    if (sleep > 500)
        sleep = 500;

    delay(sleep);
}

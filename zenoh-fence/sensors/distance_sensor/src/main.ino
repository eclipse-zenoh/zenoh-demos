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

#define URI "/factory1/room42/distance"

#define RANGER_PIN 17
#define BUZZER_PIN 16

Ultrasonic ranger(RANGER_PIN);
bool buzz_state = false;

zn_session_t *s = NULL;
zn_reskey_t *reskey = NULL;

void setup(void)
{
    Serial.begin(115200);
    while (!Serial)
        delay(10);

    WiFi.mode(WIFI_STA);
    WiFi.begin(SSID, PASS);
    while (WiFi.status() != WL_CONNECTED)
        delay(1000);
    Serial.println("Connected to WiFi!");

    // Buzzer initialization
    pinMode(BUZZER_PIN, OUTPUT);
    digitalWrite(BUZZER_PIN, buzz_state);

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
    Serial.println("Setup finished!");

    delay(1000);
}

void loop()
{
    char buf[4];
    long distance_in_cm = ranger.MeasureInCentimeters();
    Serial.print(distance_in_cm); // [0, 350]
    Serial.println(" cm");

    if (s == NULL || reskey == NULL)
        goto RET;

    // Beep / Buzzer
    buzz_state = !buzz_state;
    if (distance_in_cm > 50)
        buzz_state = false;
    digitalWrite(BUZZER_PIN, buzz_state);

    // Publish the distance
    itoa(distance_in_cm, buf, 10);
    zn_write_ext(s, *reskey, (const uint8_t *)buf, strlen(buf), Z_ENCODING_APP_INTEGER, Z_DATA_KIND_DEFAULT, zn_congestion_control_t_BLOCK);

RET:
    int sleep = distance_in_cm * 10;
    if (sleep > 500)
        sleep = 500;

    delay(sleep);
}

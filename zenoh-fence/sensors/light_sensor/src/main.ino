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

extern "C" {
    #include "zenoh-pico.h"
}

// WiFi-specific parameters
#define SSID "SSID"
#define PASS "PASS"

// Zenoh-specific parameters
#define MODE "client"
#define PEER ""

#define D_URI "/factory1/room42/distance"
#define L_R_URI "/factory1/room42/led/red"
#define L_G_URI "/factory1/room42/led/green"

#define R_PIN 25
#define G_PIN 26
#define B_PIN 27

zn_session_t *s = NULL;
zn_reskey_t *l_r_reskey = NULL;
zn_reskey_t *l_g_reskey = NULL;

int distance_in_cm = 0;

void distance_callback(const zn_sample_t *sample, const void *arg)
{
    std::string val((const char*)sample->value.val, sample->value.len);
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
    Serial.begin(115200);
    while (!Serial)
        delay(10);

    WiFi.mode(WIFI_STA);
    WiFi.begin(SSID, PASS);
    while (WiFi.status() != WL_CONNECTED)
        delay(1000);
    Serial.println("Connected to WiFi!");

    // Light/Led initialization
    pinMode(R_PIN, OUTPUT);
    pinMode(G_PIN, OUTPUT);
    pinMode(B_PIN, OUTPUT);

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

    l_r_reskey = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *l_r_reskey = zn_rid(zn_declare_resource(s, zn_rname(L_R_URI)));

    l_g_reskey = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *l_g_reskey = zn_rid(zn_declare_resource(s, zn_rname(L_G_URI)));

    zn_reskey_t d_reskey = zn_rid(zn_declare_resource(s, zn_rname(D_URI)));
    zn_subscriber_t *sub = zn_declare_subscriber(s, d_reskey, zn_subinfo_default(), distance_callback, NULL);
    if (sub == NULL)
        return;

    Serial.println("Setup finished!");

    delay(1000);
}

void loop()
{
    delay(200);
    if (s == NULL || l_r_reskey == NULL || l_g_reskey == NULL)
        goto RET;

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
    zn_write_ext(s, *l_r_reskey, (const uint8_t *)buf, strlen(buf), Z_ENCODING_APP_INTEGER, Z_DATA_KIND_DEFAULT, zn_congestion_control_t_BLOCK);

    itoa(is_green, buf, 10);
    zn_write_ext(s, *l_g_reskey, (const uint8_t *)buf, strlen(buf), Z_ENCODING_APP_INTEGER, Z_DATA_KIND_DEFAULT, zn_congestion_control_t_BLOCK);

RET:
    delay(100);
}

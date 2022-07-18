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
#define PEER "tcp/10.0.0.1:7447"

#define URI "/paris/saint-aubin/office/plants/mint/sensor/soil-moisture"

#define SENSOR_READ_PIN 34

zn_session_t *s = NULL;
zn_reskey_t *reskey = NULL;

void setup(void)
{
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

    delay(1000);
}

void loop()
{
    delay(1000);
    int value = analogRead(SENSOR_READ_PIN);
    Serial.println(value);

    char buf[5];
    itoa(value, buf, 10);

    zn_write_ext(s, *reskey, (const uint8_t *)buf, strlen(buf), Z_ENCODING_APP_INTEGER, Z_DATA_KIND_DEFAULT, zn_congestion_control_t_BLOCK);
}

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

#include <WiFi.h>
#include <BH1750.h>

extern "C" {
    #include "zenoh-pico.h"
}

// WiFi-specific parameters
#define SSID "SSID"
#define PASS "PASS"

// Zenoh-specific parameters
#define MODE "client"
#define PEER "tcp/10.0.0.1:7447"

#define URI "/paris/saint-aubin/office/rooms/jl-gb/sensor/luminosity"

BH1750 lightMeter(0x23);

zn_session_t *s = NULL;
zn_reskey_t *reskey = NULL;


void setup()
{
    Serial.begin(115200);
    Wire.begin();

    lightMeter.begin(BH1750::CONTINUOUS_HIGH_RES_MODE);
    Serial.println("Light sensor up and running");

    WiFi.mode(WIFI_STA);
    WiFi.begin(SSID, PASS);

    // Keep trying until connected
    Serial.print("Trying to connect to WiFi...");
    while (WiFi.status() != WL_CONNECTED)
    {
        Serial.print(".");
        delay(1000);
    }
    Serial.println("connected!");
    delay(1000);

    zn_properties_t *config = zn_config_default();
    zn_properties_insert(config, ZN_CONFIG_MODE_KEY, z_string_make(MODE));
    if (strcmp(PEER, "") != 0)
        zn_properties_insert(config, ZN_CONFIG_PEER_KEY, z_string_make(PEER));

    s = zn_open(config);
    if (s == NULL)
    {
        Serial.println("Error while opening zenoh session...exiting!");
        return;
    }

    znp_start_read_task(s);
    znp_start_lease_task(s);

    unsigned long rid = zn_declare_resource(s, zn_rname(URI));
    reskey = (zn_reskey_t*)malloc(sizeof(zn_reskey_t));
    *reskey = zn_rid(rid);

    zn_publisher_t *pub = zn_declare_publisher(s, *reskey);
    if (pub == NULL)
    {
        Serial.println("Error while declaring zenoh-pico publisher...exiting!");
        return;
    }

    Serial.println("Setup finished!");
}

void loop()
{
    if (lightMeter.measurementReady() == false)
        return;

    if (s == NULL || reskey == NULL)
        return;

    int lux = (int) lightMeter.readLightLevel();
    Serial.print("Light: ");
    Serial.print(lux);
    Serial.println(" lx");

    char buf[6];
    itoa(lux, buf, 10);

    zn_write_ext(s, *reskey, (const uint8_t *)buf, strlen(buf), 7, Z_DATA_KIND_DEFAULT, zn_congestion_control_t_BLOCK);


    Serial.print("Published value ");
    Serial.print(lux);
    Serial.print(" in key expr ");
    Serial.println(URI);
    delay(1000);
}


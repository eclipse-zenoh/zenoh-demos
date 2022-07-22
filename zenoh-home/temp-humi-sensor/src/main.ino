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
#include <DHT.h>

extern "C" {
    #include "zenoh-pico.h"
}

// WiFi-specific parameters
#define SSID "SSID"
#define PASS "PASS"

// Zenoh-specific parameters
#define MODE "client"
#define PEER ""

#define KEYEXPR_TEMPERATURE "paris/saint-aubin/office/rooms/jl-gb/sensor/temperature"
#define KEYEXPR_HUMIDITY "paris/saint-aubin/office/rooms/jl-gb/sensor/humidity"

#define DHT_PIN 14
DHT dht(DHT_PIN, DHT22);

z_owned_publisher_t pub_temp;
z_owned_publisher_t pub_humi;

void setup()
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

    // Initialize Zenoh Session and other parameters
    z_owned_config_t config = zp_config_default();
    zp_config_insert(z_config_loan(&config), Z_CONFIG_MODE_KEY, z_string_make(MODE));
    if (strcmp(PEER, "") != 0) {
        zp_config_insert(z_config_loan(&config), Z_CONFIG_PEER_KEY, z_string_make(PEER));
    }

    Serial.print("Detecting DHT22 sensor...");
    dht.begin();
    Serial.println("OK");

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
    Serial.print(KEYEXPR_TEMPERATURE);
    Serial.println("...");
    pub_temp = z_declare_publisher(z_session_loan(&s), z_keyexpr(KEYEXPR_TEMPERATURE), NULL);
    if (!z_publisher_check(&pub_temp)) {
        Serial.println("Unable to declare publisher for key expression!\n");
        while(1);
    }
    Serial.println("OK");
    Serial.println("Zenoh setup finished!");

    Serial.print("Declaring publisher for ");
    Serial.print(KEYEXPR_HUMIDITY);
    Serial.println("...");
    pub_humi = z_declare_publisher(z_session_loan(&s), z_keyexpr(KEYEXPR_HUMIDITY), NULL);
    if (!z_publisher_check(&pub_humi)) {
        Serial.println("Unable to declare publisher for key expression!\n");
        while(1);
    }
    Serial.println("OK");
    Serial.println("Zenoh setup finished!");

    delay(300);
}

void loop()
{
    float humi = dht.readHumidity();
    float temp = dht.readTemperature();

    Serial.print("Humidity: ");
    Serial.print(humi);
    Serial.println(" %");
    Serial.print("Temperature: ");
    Serial.print(temp);
    Serial.println(" ºC");

    char buf_humi[10];
    sprintf(buf_humi,"%f", humi);
    if (z_publisher_put(z_publisher_loan(&pub_humi), (const uint8_t *)buf_humi, sizeof(buf_humi), NULL) < 0) {
        Serial.println("Error while publishing data");
    }

    delay(100);

    char buf_temp[10];
    sprintf(buf_temp,"%f", temp);
    if (z_publisher_put(z_publisher_loan(&pub_temp), (const uint8_t *)buf_temp, sizeof(buf_temp), NULL) < 0) {
        Serial.println("Error while publishing data");
    }

    delay(1000);
}


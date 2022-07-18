# DragonBot: A TurtleBot demo using solely Zenoh stack

## **Requirements**

 * [PlatformIO](https://platformio.org)
 * [zenoh router](http://zenoh.io/docs/getting-started/quick-test/)
 * [zenoh-pico](https://github.com/eclipse-zenoh/zenoh-pico)

-----
## **Usage**

 1. Start the DragonBot:
      ```bash
      platformio run # On the first run, this is going to fail due to memcmp_P / memcpy_P in WiFi101 library, so need to run the following command:
      sed -i -e 's/memcmp_P/memcmp/g' .pio/libdeps/opencr/WiFi101/src/WiFiMDNSResponder.cpp
      sed -i -e 's/memcpy_P/memcpy/g' .pio/libdeps/opencr/WiFi101/src/WiFiMDNSResponder.cpp
      platformio run # Build again
      opencr_ld /dev/cu.usbmodemFFFFFFFEFFFF1 115200 .pio/build/opencr/firmware.bin 1 # Upload to board
      ```
 2. Start the Zenoh router:
      ```bash
      zenohd
      ```
 3. Start Ros2Teleop
      ```bash
      platformio run -t upload
      ```
 4. Move the sensor to drive the robot

See more use cases in [this blog](https://zenoh.io/blog/2022-02-02-dragonbot).


# A TurtleBot demo using solely Zenoh stack

## **Requirements**

 * [PlatformIO](https://platformio.org)
 * [zenoh router](http://zenoh.io/docs/getting-started/quick-test/)
 * [zenoh-pico](https://github.com/eclipse-zenoh/zenoh-pico)

-----
## **Usage**

 1. Start the TurtleBot:
      ```bash
      platformio run -t upload
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

See more use cases in [this blog](https://zenoh.io/blog/).


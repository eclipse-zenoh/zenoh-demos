# A zenoh-pico teleop application for ROS2 using MPU6050 sensors

## **Requirements**

 * [PlatformIO](https://platformio.org)
 * A [zenoh router](http://zenoh.io/docs/getting-started/quick-test/)
 * The [zenoh/DDS bridge](https://github.com/eclipse-zenoh/zenoh-plugin-dds#trying-it-out)
 * [zenoh-pico](https://github.com/eclipse-zenoh/zenoh-pico)
 * ROS2 [turtlesim](http://wiki.ros.org/turtlesim) (or any other robot able to receive Twist messages...)

-----
## **Usage**

 1. Start the turtlesim:
      ```bash
      ros2 run turtlesim turtlesim_node
      ```
 2. Start the zenoh router:
      ```bash
      zenohd
      ```
 3. Start the zenoh/DDS bridge:
      ```bash
      zenoh-bridge-dds
      ```
 4. Start Ros2Teleop
      ```bash
      platformio run -t upload
      ```
 5. Move the sensor to drive the robot

See more use cases in [this blog](https://zenoh.io/blog/2021-04-28-ros2-integration/).

**Notes**:

By default Ro2Teleop is configured to work with the physical turtlesim.
If you want to use a physical turtlebot3 burger,
change the TURTLESIM macro to 0 within src/main.ino.

By default Ros2Teleop publishes Twist messages on topic
`/rt/turtle1/cmd_vel` (for turtlesim) or `/rt/cmd_vel` (for turtlebot3 burger).
For other robot, change the topic within src/main.ino by redefining URI macro.

Both zenoh router and Ros2Teleop can be deployed in different networks than the robot.
Only the zenoh/DDS bridge has to run in the same network than the robot (for DDS communication via UDP multicast).
For instance, you can:
 * deploy the zenoh router in a cloud on a public IP with port 7447 open
 * configure the zenoh bridge to connect this remote zenoh router:
     ```bash
     zenoh-bridge-dds -m client -e tcp/<cloud_ip>:7447
     ```
 * configure Ros2Teleop to connect this remote zenoh router:
    ```bash
    python ros2-teleop.py -m client -e tcp/<cloud_ip>:7447
    ```

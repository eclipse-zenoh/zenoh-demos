# A zenoh Rust replay application for ROS2

## **Requirements**

 * A [Rust](https://rustup.rs/) environment
 * A [zenoh router](http://zenoh.io/docs/getting-started/quick-test/)
 * The [influxdb backend](https://github.com/eclipse-zenoh/zenoh-backend-influxdb/)
 * The [zenoh/DDS bridge](https://github.com/eclipse-zenoh/zenoh-plugin-dds#trying-it-out)
 * ROS2 [turtlesim](http://wiki.ros.org/turtlesim) (or any other robot able to receive Twist messages...)

-----
## **Usage**

### How to build

```bash
cargo build
```

### How to run

A simple reply client replaying Twists via zenoh, bridged to ROS2.

 1. Start the zenoh router:
      ```bash
      zenohd -c zenoh-influxdb.json5
      ```
 2. Start a first turtlesim:
      ```bash
      ROS_DOMAIN_ID=1 ros2 run turtlesim turtlesim_node
      ```
 3. Start the first zenoh/DDS bridge:
      ```bash
      zenoh-bridge-dds -m client -d 1
      ```
 4. Start a second turtlesim:
      ```bash
      ROS_DOMAIN_ID=2 ros2 run turtlesim turtlesim_node
      ```
 5. Start the second zenoh/DDS bridge:
      ```bash
      zenoh-bridge-dds -m client -d 2 -s /replay
      ```
 6. Teleoperate the first turtlesim by using the arrow keys:
      ```bash
      ROS_DOMAIN_ID=1 ros2 run turtlesim turtle_teleop_key
      ```
 7. Replay the first turtlesim commands to the second turtlesim
      ```bash
      ./target/debug/ros2-replay -o /replay
      ```

See more use cases in [this blog](https://zenoh.io/blog/2021-04-28-ros2-integration/).

**Notes**:

See all options accepted by Ros2TReplay with:
  ```bash
  ./target/debug/ros2-replay -h
  ```

By default ros2-replay replays Twist messages on topic `/rt/turtle1/cmd_vel` (for turtlesim).
For other robot, change the topic using the `--cmd_vel` option:
  ```bash
  ./target/debug/ros2-replay --cmd_vel /rt/my_robot/cmd_vel
  ```

Both zenoh router and Replay can be deployed in different networks than the robot. Only the zenoh/DDS bridge has to run in the same network than the robot (for DDS communication via UDP multicast).  
For instance, you can:
 * deploy the zenoh router in a cloud on a public IP with port 7447 open
 * configure the zenoh bridge to connect this remote zenoh router:
     ```bash
     zenoh-bridge-dds -m client -e tcp/<cloud_ip>:7447
     ```
 * configure Ros2Teleop to connect this remote zenoh router:
    ```bash
    ./target/debug/ros2-replay -m client -e tcp/<cloud_ip>:7447 -o /replay
    ```

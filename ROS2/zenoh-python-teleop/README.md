# A zenoh CPython teleop application for ROS2

## **Requirements**

 * Python 3.6 minimum
 * A [zenoh router](http://zenoh.io/docs/getting-started/quick-test/)
 * The [zenoh/DDS bridge](https://github.com/eclipse-zenoh/zenoh-plugin-dds#trying-it-out)
 * [zenoh-python](https://github.com/eclipse-zenoh/zenoh-python): install it with `pip install eclipse-zenoh`.
 * [pycdr](https://pypi.org/project/pycdr/): install it with `pip install pycdr`.
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
      python ros2-teleop.py
      ```
 5. Use the arrows keys to drive the robot

See more use cases in [this blog](https://zenoh.io/blog/2021-04-28-ros2-integration/).

**Notes**:

See all options accepted by Ros2Teleop with:
  ```bash
  python ros2-teleop.py -h
  ```

By default Ros2Teleop publishes Twist messages on topic `/rt/turtle1/cmd_vel` (for turtlesim).
For other robot, change the topic using the `--cmd_vel` option:
  ```bash
  python ros2-teleop.py --cmd_vel /rt/my_robot/cmd_vel
  ```

Both zenoh router and Teleop can be deployed in different networks than the robot. Only the zenoh/DDS bridge has to run in the same network than the robot (for DDS communication via UDP multicast).  
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



# Some examples of using Zenoh REST API for ROS2

## **Requirements**

* A [zenoh router](http://zenoh.io/docs/getting-started/quick-test/)
* The [zenoh-ros2dds bridge](https://github.com/eclipse-zenoh/zenoh-plugin-ros2dds)
* [jscdr](https://github.com/atolab/jscdr)
   (automatically retrieved by your Web browser)
* ROS2 [turtlesim](http://wiki.ros.org/turtlesim) (or any other robot able to receive Twist messages...)

-----

## **Usage**

### ros2-teleop.html

A simple teleop web page allowing to publish Twists and to subscribe to Logs
via the zenoh REST API, bridged to ROS2.

1. Start the turtlesim:

     ```bash
     ros2 run turtlesim turtlesim_node
     ```

2. Start the zenoh/DDS bridge, activating its REST API:

     ```bash
     zenoh-bridge-ros2dds --rest-http-port 8000
     ```

3. Open the `ros2-teleop.html` file in a Web browser

4. In this page:

     * If needed, adapt the "Zenoh REST API" URL to the host where your bridge is running
     * Keep pressing the arrow buttons (or arrows on your keyboard) to publish Twist messages (a STOP Twist with 0 speed is published when released).
     * All the messages published on "rosout" via ROS2 will be displayed in the bottom box.

### ros2-tb3-teleop.html

A simple teleop web page allowing to publish Twists and to subscribe to Logs
via the zenoh REST API, bridged to ROS2 for Turtlebot3 on Gazebo.

1. Start the Turtlebot3 launch file:
Please clone the turtlebot3_simulations package beforehand.

     ```bash
     RMW_IMPLEMENTATION=rmw_cyclonedds_cpp ros2 launch turtlebot3_gazebo turtlebot3_wolrd.launch.py
     ```

2. Start the zenoh/DDS bridge, activating its REST API:

     ```bash
     zenoh-bridge-ros2dds --rest-http-port 8000
     ```

3. Open the `ros2-tb3-teleop.html` file in a Web browser

⚠️ Note: this demo depends on the Ros2 Middleware implementation being set to CycloneDDS
Installation instructions for CycloneDDS below:
https://docs.ros.org/en/humble/Installation/DDS-Implementations/Working-with-Eclipse-CycloneDDS.html
Once installed the middleware env var must be set :
`RMW_IMPLEMENTATION=rmw_cyclonedds_cpp`
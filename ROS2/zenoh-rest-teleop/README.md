# Some examples of using Zenoh REST API for ROS2

## **Requirements**

 * A [zenoh router](http://zenoh.io/docs/getting-started/quick-test/)
 * The [zenoh/DDS bridge](https://github.com/eclipse-zenoh/zenoh-plugin-dds#trying-it-out)
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
      zenoh-bridge-dds --rest-plugin
      ```
 5. Open the `ros2-teleop.html` file in a Web browser
 6. In this page:
     - If needed, adapt the "URL of your zenoh-bridge-dds" to the host where your bridge is running
     - Click on "Subscribe" button
     - All the messages published on "/rt/rosout" via ROS2 should be displayed in the bottom box.


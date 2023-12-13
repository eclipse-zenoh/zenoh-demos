# Turtlebot3 Demos

## Demos in this directory

* **[zdrive-python](./zdrive-python)**: An application that listens for cmd_vel
  messages from zenoh and drive motors accordingly.
* **[zlidar-rust](./zlidar-rust)**: An application that publishes LiderScan
  messages to zenoh.
* **[zturtle-rust](./zturtle-rust)**: An application that listens for cmd_vel
  messages from zenoh and drive motors accordingly, and publishes images from
  the camera to zenoh.
* **[zturtle-pythont](./zturtle-pythont)**: An application that listens for
  cmd_vel messages from zenoh and drive motors accordingly, and publishes images
  from the camera to zenoh.

Those demos are designed to be run on a turtlebot and to interoperate with :

* **[ROS2 demos](../ROS2)**
* **[zcam demos](../computer-vision/zcam)**

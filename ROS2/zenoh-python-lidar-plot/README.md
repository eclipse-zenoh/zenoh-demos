# A zenoh Rust lidar display application for ROS2

## **Requirements**

* Python 3.6 minimum
* A [zenoh router](http://zenoh.io/docs/getting-started/quick-test/)
* The [zenoh/DDS bridge](https://github.com/eclipse-zenoh/zenoh-plugin-dds#trying-it-out)
* [zenoh-python](https://github.com/eclipse-zenoh/zenoh-python): install it with `pip install eclipse-zenoh`.
* [pycdr2](https://pypi.org/project/pycdr2/): install it with `pip install pycdr2`.
* [matplotlib](https://pypi.org/project/matplotlib/) and [numpy](https://pypi.org/project/numpy/): install them with `pip install numpy matplotlib`.
* ROS2 [turtlesim](http://wiki.ros.org/turtlesim) (or any other robot able to send LaserScans...)

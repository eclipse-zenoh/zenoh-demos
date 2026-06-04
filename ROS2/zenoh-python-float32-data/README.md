# A zenoh Python Float32 data for ROS2

## **Requirements**

* Python 3.6 minimum
* A [zenoh router](http://zenoh.io/docs/getting-started/quick-test/)
* The [zenoh/DDS bridge](https://github.com/eclipse-zenoh/zenoh-plugin-dds#trying-it-out)
* [zenoh-python](https://github.com/eclipse-zenoh/zenoh-python): install it with `pip install eclipse-zenoh`.
* [pycdr2](https://pypi.org/project/pycdr2/): install it with `pip install pycdr2`.

-----

## **Usage**

1. Start the zenoh/DDS bridge:

    ```bash
    zenoh-bridge-dds
    ```

2. Start Ros2Float32

    ```bash
    python ros2-float32-data.py
    ```

3. Start Rviz2 and Setting Plotter2D

    ```bash
    python ros2-float32-data.py
    ```

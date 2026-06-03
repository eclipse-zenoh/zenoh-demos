# zlidar-python

A demo for turtlebot3 robots using [zenoh](http://zenoh.io) python.

Note: this demo only works with turtlebot3 LDS-02 lidars

## Running zlidar-python demo

On the turtlebot, run the zlidar app:

```bash
python3 zlidar.py
```

Plot the laser scans using zenoh-demos/ROS2/zenoh-python-lidar-plot:

```bash
python3 ros2-lidar-plot.py
```

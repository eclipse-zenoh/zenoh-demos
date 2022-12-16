# zdrive-python
A demo for turtlebot3 robots using [zenoh](http://zenoh.io) python.

## Dependencies
```
pip3 install dynamixel-sdk pycdr2 eclipse-zenoh-nightly
```

## Running zdrive-python demo

On the turtlebot, run the zdrive app:
```
python3 zdrive.py
```

Teleoperate the robot using zenoh-demos/ROS2/zenoh-python-teleop or zenoh-demos/ROS2/zenoh-rust-teleop:
```
python3 ros2-teleop.py -a 200.0 -x 20.0
```





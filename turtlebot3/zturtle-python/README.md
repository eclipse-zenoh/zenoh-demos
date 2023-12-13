# zturtle-python

A demo for turtlebot3 robots using [zenoh](http://zenoh.io) python.

## Dependencies

```console
pip3 install opencv-python imutils dynamixel-sdk pycdr2 eclipse-zenoh-nightly
```

## Running zturtle-python demo

Run the zenoh router on a remote host:

```console
zenohd
```

On the turtlebot, edit endpoints.json to point to the running zenoh router.

On the turtlebot, run the zturtle app:

```console
python3 zturtle.py -m client -e endpoints.json
```

Display the video stream using zenoh-demos/computer-vision/zcam/zcam-python or zenoh-demos/computer-vision/zcam/zcam-rust:

```console
python3 zdisplay.py -k 'rt/*/cams/*' -m client -e tcp/127.0.0.1:7447
```

Teleoperate the robot using zenoh-demos/ROS2/zenoh-python-teleop or zenoh-demos/ROS2/zenoh-rust-teleop:

```console
python3 ros2-teleop.py -m client -e tcp/127.0.0.1:7447 --cmd_vel=rt/turtle1/cmd_vel -a 200.0 -x 20.0
```

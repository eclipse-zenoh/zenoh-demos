# zturtle-rust

A demo for turtlebot3 robots using [zenoh](http://zenoh.io) rust.

## Dependencies

In order to build and run zcam-rust you need to have [Rust](rust-lang.org) and
[OpenCV](http://opencv.org) installed on your machine.

- [Rust](http://rust-lang.org) installation instructions are available [here](https://www.rust-lang.org/tools/install)
- [OpenCV](http://opencv.org) installation instructions are available [here](https://docs.opencv.org/trunk/df/d65/tutorial_table_of_content_introduction.html).

## Building zturtle-rust demo

```bash
git clone https://github.com/eclipse-zenoh/zenoh-demos.git
cd turtlebot3/zturtle-rust
cargo build --release
```

## Running zturtle-rust demo

Run the zenoh router on a remote host:

```console
zenohd
```

On the turtlebot, edit endpoints.json to point to the running zenoh router.

On the turtlebot, run the zturtle app:

```console
zturtle -m client -e endpoints.json
```

Display the video stream using zenoh-demos/computer-vision/zcam/zcam-rust or zenoh-demos/computer-vision/zcam/zcam-python:

```console
zdisplay -k 'rt/*/cams/*' -m client -e tcp/127.0.0.1:7447
```

Teleoperate the robot using zenoh-demos/ROS2/zenoh-rust-teleop or zenoh-demos/ROS2/zenoh-teleop-python:

```console
ros2-teleop -m client -e tcp/127.0.0.1:7447 --cmd_vel=rt/turtle1/cmd_vel -a 100.0 -x 20.0
```

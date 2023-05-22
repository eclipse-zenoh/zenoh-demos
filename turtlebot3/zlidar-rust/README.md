# zlidar-rust
A demo for turtlebot3 robots using [zenoh](http://zenoh.io) rust.

## Dependencies
- [Rust](http://rust-lang.org) installation instructions are available [here](https://www.rust-lang.org/tools/install)

## Building zlidar-rust demo
```bash
git clone https://github.com/eclipse-zenoh/zenoh-demos.git
cd turtlebot3/zlidar-rust
cargo build --release
```
## Running zlidar-rust demo

On the turtlebot, run the zlidar app:
```
./target/release/zlidar
```

Plot the laser scans using zenoh-demos/ROS2/zenoh-python-lidar-plot:
```
python3 ros2-lidar-plot.py
```





# Zenoh Flow powered turtlebot3

This folder contains the whole set of operators needed to fully teleoperate a Turtlebot3 with Zenoh Flow

:warning: **You need a Joypad connected to your machine in order to run this demo**

## Build

All the components can be build with `cargo`

### Workstation components
```bash
$ cargo build --release -p lidar-sink -p source-gamepad -p operator-twist -p robot-sink
```


### Robot components
```
$ rustup target add aarch64-unknown-linux-gnu
$ cargo build --release aarch64-unknown-linux-gnu -p source-tick -p tb3 -p source-lidar
```

#### Copy the components to the robot

```
$ scp -r ./target/aarch64-unknown-linux-gnu/release/\*.so ubuntu@<robot ip address>:~/zf-bot
$ scp -r ./turtlebot3.yaml ubuntu@<robot ip address>:~/zf-bot
```


### Building the Zenoh runtime

```bash
$ cd ~
$ git clone https://github.com/ZettaScaleLabs/zenoh-flow-examples
$ cargo build --release -p runtime
```
#### Building and copy to the robot

```bash
$ cargo build --release -p runtime --target aarch64-unknown-linux-gnu
$  scp -r ./target/aarch64-unknown-linux-gnu/release/runtime ubuntu@<robot ip address>:~/zf-bot
```

## Run it

:warning: **If you are using Linux or Windows you need to change the extension of all the operators mapped to the workstation in the descriptor file.**


### Workstation

```bash
$ ~/zenoh-flow-examples/target/release/runtime -g turtlebot3.yaml -r ws
```

### Robot

```bash
$ ssh ubuntu@<robot ip address>
$ ~/zf-bot/runtime -g ~/zf-bot/turtlebot3.yaml -r robot
```

You can now teleoperate the robot using a the triggers and analog stick of a joypad connected to the workstation!
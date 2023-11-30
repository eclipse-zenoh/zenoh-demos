<img src="https://raw.githubusercontent.com/eclipse-zenoh/zenoh/master/zenoh-dragon.png" height="150">

[![Discussion](https://img.shields.io/badge/discussion-on%20github-blue)](https://github.com/eclipse-zenoh/roadmap/discussions)
[![Discord](https://img.shields.io/badge/chat-on%20discord-blue)](https://discord.gg/2GJ958VuHs)

# Eclipse Zenoh

The Eclipse Zenoh: Zero Overhead Pub/sub, Store/Query and Compute.

Zenoh (pronounce _/zeno/_) unifies data in motion, data at rest and computations. It carefully blends traditional pub/sub with geo-distributed storages, queries and computations, while retaining a level of time and space efficiency that is well beyond any of the mainstream stacks.

Check the website [zenoh.io](http://zenoh.io) and the [roadmap](https://github.com/eclipse-zenoh/roadmap) for more detailed information.

-------------------------------

## Description

<!-- TODO: Add pictures -->

**zenoh-tetris**: a networked two-player
[Tetris](https://en.wikipedia.org/wiki/Tetris) implementation written with Zenoh
and Rust. The game follows the client-server model. A server manages the game
state, publishes it and subscribes to player input. While a client subscribes to
the game state, renders it and publishes player input. Thus, clients can play
against each other from potentially different network hosts.

**zenoh-shamir**: illustrates [Shamir's secret
sharing](https://en.wikipedia.org/wiki/Shamir%27s_Secret_Sharing) by splitting a
secret into 'shares' and storing them on distinct, interconnected Zenoh routers.
Another Zenoh node implements a
[Queryable](https://zenoh.io/docs/manual/abstractions/#queryable) which collects
all shares into the original secret.

**zenoh-pico-dds/{helloworld,roundtrip}**: utilize
[zenoh-plugin-dds](https://github.com/eclipse-zenoh/zenoh-plugin-dds) to connect
a [CycloneDDS](https://github.com/eclipse-cyclonedds/cyclonedds) node and a
Zenoh node, enabling either node to publish, or subscribe to, a given
resource/topic. Data is exchanged using the
[CDR](https://en.wikipedia.org/wiki/Common_Data_Representation) representation,
as implemented in CycloneDDS. The `helloworld` demo consists of one publisher
and one subscriber, while the `rountrip` demo implements a simple ping-pong
scheme.

**turtlebot/zturtle-{python,rust}**: a [TurtleBot
3](https://en.wikipedia.org/wiki/TurtleBot) teleoperation application leveraging
Zenoh. It subscribes to
[Twist](https://docs.ros.org/en/noetic/api/geometry_msgs/html/msg/Twist.html)
messages to control the velocity of the TurtleBot's wheels, publishes a camera
feed, and whenever its Wi-Fi network changes, it connects to a new peer/router
(e.g. to maintain geo-proximity). The Rust and Python implementations are
largely equivalent.

**turtlebot/zdrive-python**: a stripped down version of
**turtlebot/zturtle-python** providing only teleoperation (i.e. no camera feed
and no dynamic peer/router re-connections). This demo can be less demanding on
low-bandwidth networks.

**plotting**: illustrates how Zenoh can be used to publish metrics to a
dashboard. Three subscriber frontends are provided: a Python script using
[Matplotlib](https://matplotlib.org/), a browser client and a
[Freeboard](https://freeboard.github.io) configuation.

**computer-vision/zcam/zcam-{python,rust,rest}**: consists of two Zenoh nodes: a
publisher capturing a camera video stream and a subscriber displaying said video
stream. Both of the Python and Rust implementations use
[OpenCV](https://opencv.org/) to encode and decode data.

**computer-vision/face-recog**: a system of four Zenoh nodes communicating with
each other to (1) capture a camera video stream, detect (2) then recoginize (3)
faces within it and finally display (4) the results. The purpose being
identifying faces based given a database of pictures.

**distributed-web**
: illustrates how Zenoh can be used to host a geo-distributed
web page by splitting page content across multiple routers each running separate
storage backends. This demo also demonstrates the use of dynamic router
configuration through Zenoh's [REST API](https://zenoh.io/docs/apis/rest/).

**ROS2/zenoh-pico-teleop-gyro**: utilizes an
[ESP32](https://en.wikipedia.org/wiki/ESP32) board and a
[gyroscope](https://en.wikipedia.org/wiki/Gyroscope) sensor to publish
[Twist](https://docs.ros.org/en/noetic/api/geometry_msgs/html/msg/Twist.html)
teleoperation messages over Zenoh
[Pico](https://github.com/eclipse-zenoh/zenoh-pico). This is done by mapping 3D
rotations to velocity vectors and can for example be used to operate a
[TurtleBot 3](https://en.wikipedia.org/wiki/TurtleBot) using hand gestures.

**ROS2/zenoh-python-lidar-plot**: a Zenoh that subscribes to
[LaserScan](http://docs.ros.org/en/melodic/api/sensor_msgs/html/msg/LaserScan.html)
messages published by a robot equiped with a
[Lidar](https://en.wikipedia.org/wiki/Lidar) sensor. Then, using
[Matplotlib](https://matplotlib.org/), it constructs a 2D map of nearby
obstacles and environment boundaries (e.g. walls of a room).

**ROS2/zenoh-{python,rust,rest}-teleop**: Zenoh nodes that publish
[Twist](https://docs.ros.org/en/noetic/api/geometry_msgs/html/msg/Twist.html)
teleoperation messages by reading keyboard input (i.e. arrow keys). Both of the
Rust and Python implementations are terminal applications while the REST version
runs on the browser.

**zenoh-home/{light,soil,temp-humi}-sensor**: Zenoh nodes running Zenoh
[Pico](https://github.com/eclipse-zenoh/zenoh-pico) on an
[ESP32](https://en.wikipedia.org/wiki/ESP32) board which publish sensor data
either from an [ambient light
sensor](https://learn.adafruit.com/adafruit-bh1750-ambient-light-sensor/overview),
a soil moisture sensor or a [temporature & humidity
sensor](https://learn.adafruit.com/dht).

**zenoh-android/ZenohApp**: an Android application written using Zenoh's Kotlin
bindings. It can declare a subscriber, a publisher or a queryable, as well as
perform a PUT, GET or DELETE operation.

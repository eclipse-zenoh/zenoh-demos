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

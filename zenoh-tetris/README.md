## Network tetris game on Zenoh

The pair tetris game powered with Zenoh. 

Game rules: 

- Normal tetris rules
- When player removes line, randomly filled line pops from the bottom of the opponnent's glass.

Usage:

- Local play

    - Run local game with
      ```sh
      cargo run --bin hot_seat
      ```

- Network play

    - Run one or more servers with
      ```bash
      cargo run --bin server
      ```

        The game will immediately start, server name will be displayed
        ```
        │                    │ ┌────────┐    ┌────────┐    │                    │
        │                    │ │  []    │    │  [][]  │    │                    │
        │                    │ │[][][]  │    │[][]    │    │                    │
        │                    │ │        │    │        │    │                    │
        │                    │ │        │    │        │    │                    │
        │                    │ └────────┘    └────────┘    │                    │
        │                    │                             │                    │
        │                    │ Server:       PLAYER        │                    │
        │                    │                             │                    │
        │                    │ avocado       <- Move Left  │                    │
        │                    │               -> Move Right │                    │
        │                    │               ^ Rotate      │                    │
        │      [][][][]      │               v Accelerate  │          []        │
        │                    │               Space: Drop   │      [][][]        │
        │                    │                             │                    │
        │                    │                             │                    │
        │                    │                             │                    │
        │                    │                             │                    │
        │                    │                             │                    │
        │                    │                             │                    │
        └────────────────────┘                             └────────────────────┘
        ```

    - Run client with
        ```bash
        cargo run --bin client
        ```

        You will be asked to select server to play:
        ```
        Select server:
        0: ALL at tetris/*
        1: avocado at tetris/5c68ee21-e5da-4c07-9a1a-45b9657889fe
        ```

        Select server and play. Selecting 'ALL' means sending the client's
        actions to and receiving the picture from all servers at the same time.

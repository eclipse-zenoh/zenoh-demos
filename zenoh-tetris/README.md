## Network tetris game on Zenoh

The pair tetris game powered with Zenoh. 

Gamwe rules: 

- Normal tetris rules
- When player removes line, randomly filled line pops from bottom in opponnent's glass.

Usage:

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
0: tetris/* at ALL
1: tetris/5c68ee21-e5da-4c07-9a1a-45b9657889fe at avocado
```

Select server and start play. Selecting 'ALL' means sending you key presses and receiving picture from to all servers at the same time.

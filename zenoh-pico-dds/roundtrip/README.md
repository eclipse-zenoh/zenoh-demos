# RoundTrip
The Roundtrip example allows the measurement of roundtrip duration when sending and receiving back a single message

This example is compatible with CycloneDDS' [roundtrip example](https://github.com/eclipse-cyclonedds/cyclonedds/tree/0.10.2/examples/roundtrip) (sources copied and built here for convenience).

## Prerequisite:

 - CycloneDDS installed and its `idlc` code generator accessible in the `${PATH}`
 - CMake 2.14+
 - [zenoh-bridge-dds](https://github.com/eclipse-zenoh/zenoh-plugin-dds)

## How to build:

```bash
mkdir build
cd build
cmake ..
make
```

## Result of build:

 - **`RoundtripPing`**: CycloneDDS' ping example.

 - **`RoundtripPong`**: CycloneDDS' pong example.

 - **`z_ping`**: A zenoh-pico equivalent to RoundtripPing, sending a ping message and waiting for a poing message in return.  
   This zenoh-pico application is in **client** mode and tries to establish
   a session with a router or a `zenoh-bridge-dds` on specified `zenoh_locator`.  
   Usage:
      ```bash
      z_ping <payloadSize (bytes, 0 - 100M)> <numSamples (0 = infinite)> <timeOut (seconds, 0 = infinite)> [zenoh_locator (default: tcp/127.0.0.1)]
      ```

 - **`z_pong`**: A zenoh-pico equivalent to RoundtripPong, waiting for ping messages and sending the same message back as a pong.  
   This zenoh-pico application is in **client** mode and tries to establish
   a session with a router or a `zenoh-bridge-dds` on specified `zenoh_locator`.  
   Usage:
      ```bash
      z_pong [zenoh_locator (default: tcp/127.0.0.1)]
      ```

## Example of usage (on localhost):

 - zenoh-pico ping and CycloneDDS pong:
    - run: `zenoh-bridge-dds -l tcp/0.0.0.0:7447`
    - run: `RoundtripPong`
    - run: `z_ping 8 0 0`

 - CycloneDDS ping and zenoh-pico pong:
    - run: `zenoh-bridge-dds -l tcp/0.0.0.0:7447`
    - run: `z_pong`
    - run: `RoundtripPing 8 0 0`

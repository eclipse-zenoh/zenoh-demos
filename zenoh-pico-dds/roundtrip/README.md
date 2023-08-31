# RoundTrip
This example is compatible with CycloneDDS' [roundtrip example](https://github.com/eclipse-cyclonedds/cyclonedds/tree/master/examples/roundtrip).

It shows how to leverage the `idlc` compiler and the CDR encoding/decoding library from CycloneDDS in a zenoh-pico application.

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

 - **`z_ping`**: Sends a message to pong and waits for its return.  
   This zenoh-pico application is in **client** mode and tries to establish
   a session with a router or a `zenoh-bridge-dds` on specified `zenoh_locator`.  
   Usage:
      ```bash
      z_ping <payloadSize (bytes, 0 - 100M)> <numSamples (0 = infinite)> <timeOut (seconds, 0 = infinite)> [zenoh_locator (default: tcp/127.0.0.1)]
      ```

 - **`z_pong`**: Waits for messages from ping and sends the same message back.  
   This zenoh-pico application is in **client** mode and tries to establish
   a session with a router or a `zenoh-bridge-dds` on specified `zenoh_locator`.  
   Usage:
      ```bash
      z_pong [zenoh_locator (default: tcp/127.0.0.1)]
      ```

## Example of usage (on localhost):

 - zenoh-pico ping and CycloneDDS pong:
    - run: `zenoh-bridge-dds -l tcp/0.0.0.0:7447`
    - run: `pong`
    - run: `z_ping 8 0 0`

 - CycloneDDS ping and zenoh-pico pong:
    - run: `zenoh-bridge-dds -l tcp/0.0.0.0:7447`
    - run: `z_pong`
    - run: `ping 8 0 0`

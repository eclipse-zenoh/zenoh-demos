# HelloWorld
This example is compatible with CycloneDDS' [helloworld example](https://github.com/eclipse-cyclonedds/cyclonedds/tree/master/examples/helloworld).

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

 - **`z_sub_cdr`**: an example of zenoh-pico subscriber that decodes the received payloads from CDR as a `HelloWorldData_Msg` (see definition in `idl/HelloWorldData.idl` file).  
   Options:

     - `-k <keyexpr>`: the key expression to subscribe (equal to the DDS topic when routed by `zenoh-bridge-dds`). Default: `HelloWorldData_Msg`
     - `-e <endpoint>`: a Zenoh endpoint to establish the connection with (e.g.: `tcp/192.168.2.3:7447`). Default: zenoh-pico tries to discover a Zenoh router via UDP multicast.
     - `-m <client|peer>`: the mode in which zenoh-pico is running. Default: `client`

 - **`z_pub_cdr`**: an example of zenoh-pico publisher that encodes the payloads to CDR from a `HelloWorldData_Msg` (see definition in `idl/HelloWorldData.idl` file).  
   Options:

     - `-k <keyexpr>`: the key expression to subscribe (equal to the DDS topic when routed by `zenoh-bridge-dds`). Default: `HelloWorldData_Msg`
     - `-e <endpoint>`: a Zenoh endpoint to establish the connection with (e.g.: `tcp/192.168.2.3:7447`). Default: zenoh-pico tries to discover a Zenoh router via UDP multicast.
     - `-m <client|peer>`: the mode in which zenoh-pico is running. Default: `client`

## Example of usage (on localhost):

 - zenoh-pico publisher and CycloneDDS subscriber:
    - run: `zenoh-bridge-dds -l tcp/0.0.0.0:7447`
    - run: `HelloworldSubscriber`
    - run: `z_pub_cdr -e tcp/127.0.0.1:7447`

 - CycloneDDS publisher and zenoh-pico subscriber:
    - run: `zenoh-bridge-dds -l tcp/0.0.0.0:7447`
    - run: `z_sub_cdr -e tcp/127.0.0.1:7447`
    - run: `HelloworldPublisher`

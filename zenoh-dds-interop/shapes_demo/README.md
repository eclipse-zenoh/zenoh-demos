# shapes-demo

A program to test the interoperability of [zenoh-plugin-dds](https://github.com/eclipse-zenoh/zenoh-plugin-dds) against various DDS implementations.

## Dependencies


- [Rust](http://rust-lang.org) installation instructions are available [here](https://www.rust-lang.org/tools/install)
- [zenoh-plugin-dds](https://github.com/eclipse-zenoh/zenoh-plugin-dds)
- A DDS Shapes Demo application, using e.g. [RustDDS](https://atostek.com/en/services/rust-dds/) that has a built-in [Shapes Demo](https://github.com/jhelovuo/RustDDS/tree/master/examples/shapes_demo).

## Building Shapes Demo

Build the Zenoh-DDS bridge

```bash
git clone https://github.com:eclipse-zenoh/zenoh-plugin-dds.git
cd zenoh-plugin-dds
cargo build --release
```

Build the shapes demo for Zenoh

```bash
git clone https://github.com/eclipse-zenoh/zenoh-demos.git
cd zenoh-demos/zenoh-dds-interop/shapes_demo
cargo build --release
```

Build RustDDS and its Shapes Demo

```bash
git clone https://github.com:jhelovuo/RustDDS.git
cd RustDDS
cargo build --examples
```


## Running the demo

Using separate consoles, start the three applications.

Start the zenoh-plugin-dds in standalone bridge mode:

```bash
cd zenoh-plugin-dds
cargo run
```

Start RustDDS Shapes Demo to subscribe on topic `Square`:
```bash
cd RustDDS
cargo run --example=shapes_demo -- -S -t Square
```

Start Zenoh Shapes Demo to publish on `Square`:

```bash
cd zenoh-demos/zenoh-dds-interop/shapes_demo
cargo run  -- -P -t Square
```

Expected result: Zenoh Shapes demo receives what DDS Shapes Demo publishes. Startup order of the programs should not matter.

## Things to try out

Swap the arguments -P and -S between programs to test the other direction.

Other DDS implementations, e.g. 
[FastDDS](https://github.com/eProsima/Fast-DDS), 
[CycloneDDS](https://cyclonedds.io/), or 
[RTI Connext](https://www.rti.com/products) 
are also expected to work with their respective Shapes Demo programs.

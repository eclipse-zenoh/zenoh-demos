# zcam-rust -- Streaming video with zenoh-rust
This is a simple application that shows how to stream HD Video with [zenoh](http://zenoh.io)

## Dependencies
In order to build and run zcam-rust you need to have [Rust](rust-lang.org) and [OpenCV]() installed on your machine. 
- [Rust](http://rust-lang.org) installation instructions are available [here](https://www.rust-lang.org/tools/install)
- [OpenCV](http://opencv.org) installation instructions are available [here](https://docs.opencv.org/trunk/df/d65/tutorial_table_of_content_introduction.html).

## Building and Running zcam-rust
To get and build zcam-rust do the following:

```bash
git clone https://github.com/eclipse-zenoh/zenoh-demos.git
cd computer-vision/zcam/zcam-rust
cargo build --release
```

Once build you can run it as follows:

```bash
./target/release/zcapture -k /demo/video/yourname
./target/release/zdisplay -k /demo/video/*
```

# zcam-rust -- Streaming video with zenoh-rust

This is a simple application that shows how to stream HD Video with [zenoh](http://zenoh.io)

## Dependencies

In order to build and run zcam-rust you need to have [Rust](rust-lang.org) and [OpenCV](http://opencv.org) installed on your machine.

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
./target/release/zcapture -k 'demo/zcam/yourname'
./target/release/zdisplay -k 'demo/zcam/*'
```

Or with face detection (fully zero-copy):

```bash
./target/release/zcapture -k 'demo/zcam/yourname'
./target/release/zhaar -k 'demo/zcam/yourname' --key-pub 'demo/zcam/yourname/facedetect'
./target/release/zdisplay -k 'demo/zcam/yourname/facedetect'
```

Or with face detection and jpg encoding/decoding:

```bash
./target/release/zcapture -k 'demo/zcam/yourname'
./target/release/zhaar -k 'demo/zcam/yourname' --key-pub  'demo/zcam/yourname/facedetect'
./target/release/zencode -k 'demo/zcam/yourname/facedetect' --key-pub  'demo/zcam/yourname/facedetect/encoded'
./target/release/zdecode -k 'demo/zcam/yourname/facedetect/encoded' --key-pub  'demo/zcam/yourname/facedetect/decoded'
./target/release/zdisplay -k 'demo/zcam/yourname/facedetect/decoded'
```

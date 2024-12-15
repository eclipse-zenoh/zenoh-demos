
# zcam-rs-python -- Streaming video with zenoh-realsense-python

This is a simple application that shows how to stream Intel RealSense Stream with [zenoh](http://zenoh.io)

## Dependencies

```bash
pip3 install eclipse-zenoh opencv-python numpy imutils
sudo apt-get install librealsense2-dkms librealsense2-utils librealsense2-dev librealsense2-dbg
pip install pyrealsense2
```

## Running zcam-python

```bash
python3 zcapture_rs.py
python3 zdisplay_rs.py
```

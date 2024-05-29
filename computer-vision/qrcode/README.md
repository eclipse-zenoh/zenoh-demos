# Eclipse Zenoh QRCode Detection Demo

This is a relatively simple demo that shows how zenoh can be used to do QRCode detection and decoding as well as notification of detected QRCodes.

## Pre-requisite

Python 3, pip3 and the zenoh-python api.
Install the required python modules:

```bash
pip3 install jsonschema jsonpickle argcomplete imutils opencv-python opencv-contrib-python eclipse-zenoh
```

## Step I -- Run the video capture component

This component reads frames from the camera and publishes them to zenoh.

```bash
python3 capture_video.py
```

## Step II -- Run the QRCode detection and decoding component

This component subscribes to video frames from the video capture component, detects and decode QRCodes and publishes the QRCodes info and coordinates to zenoh.

```bash
python3 detect_and_decode.py
```

## Step III -- Run the display component

This component subscribes to frames from the video capture component, and to QRCodes info and coordinates from the QRCode detection and decoding component on zenoh and displays them.

```bash
python3 display_video.py
```

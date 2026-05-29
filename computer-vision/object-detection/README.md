# Eclipse Zenoh Object Detection Demo

This is a relatively simple demo that shows how zenoh can be used to do object detection as well as notification of detected objects.

## Pre-requisite

Python 3, pip3 and the zenoh-python api.
Install the required python modules:

```bash
pip3 install jsonschema jsonpickle argcomplete imutils opencv-python opencv-contrib-python ultralytics eclipse-zenoh
```

## Step I -- Run the video capture component

This component reads frames from the camera and publishes them to zenoh.

```bash
python3 capture_video.py
```

## Step II -- Run the objects detection component

This component subscribes to video frames from the video capture component, detects objects and publishes the objects info and coordinates to zenoh.

```bash
python3 detect_objects.py
```

## Step III -- Run the display component

This component subscribes to frames from the video capture component, and to objects info and coordinates from the objects detection component on zenoh and displays them.

```bash
python3 display_video.py
```

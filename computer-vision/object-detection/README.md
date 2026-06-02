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

## List of detected objects

- 0: 'person'
- 1: 'bicycle'
- 2: 'car'
- 3: 'motorcycle'
- 4: 'airplane'
- 5: 'bus'
- 6: 'train'
- 7: 'truck'
- 8: 'boat'
- 9: 'traffic light'
- 10: 'fire hydrant'
- 11: 'stop sign'
- 12: 'parking meter'
- 13: 'bench'
- 14: 'bird'
- 15: 'cat'
- 16: 'dog'
- 17: 'horse'
- 18: 'sheep'
- 19: 'cow'
- 20: 'elephant'
- 21: 'bear'
- 22: 'zebra'
- 23: 'giraffe'
- 24: 'backpack'
- 25: 'umbrella'
- 26: 'handbag'
- 27: 'tie'
- 28: 'suitcase'
- 29: 'frisbee'
- 30: 'skis'
- 31: 'snowboard'
- 32: 'sports ball'
- 33: 'kite'
- 34: 'baseball bat'
- 35: 'baseball glove'
- 36: 'skateboard'
- 37: 'surfboard'
- 38: 'tennis racket'
- 39: 'bottle'
- 40: 'wine glass'
- 41: 'cup'
- 42: 'fork'
- 43: 'knife'
- 44: 'spoon'
- 45: 'bowl'
- 46: 'banana'
- 47: 'apple'
- 48: 'sandwich'
- 49: 'orange'
- 50: 'broccoli'
- 51: 'carrot'
- 52: 'hot dog'
- 53: 'pizza'
- 54: 'donut'
- 55: 'cake'
- 56: 'chair'
- 57: 'couch'
- 58: 'potted plant'
- 59: 'bed'
- 60: 'dining table'
- 61: 'toilet'
- 62: 'tv'
- 63: 'laptop'
- 64: 'mouse'
- 65: 'remote'
- 66: 'keyboard'
- 67: 'cell phone'
- 68: 'microwave'
- 69: 'oven'
- 70: 'toaster'
- 71: 'sink'
- 72: 'refrigerator'
- 73: 'book'
- 74: 'clock'
- 75: 'vase'
- 76: 'scissors'
- 77: 'teddy bear'
- 78: 'hair drier'
- 79: 'toothbrush'

# zcam-python -- Streaming video with zenoh-python
This is a simple application that shows how to stream HD Video with [zenoh](http://zenoh.io)

## Dependencies

```bash
pip3 install eclipse-zenoh opencv-python numpy
```

## Running zcam-python

```bash
python3 zcapture.py -k demo/zcam/yourname
python3 zdisplay.py -k demo/zcam/*
```

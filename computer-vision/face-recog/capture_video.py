import argparse
from imutils.video import VideoStream
import imutils
import time
import io
import cv2
import json
import random
import zenoh
import binascii
import numpy as np

parser = argparse.ArgumentParser(
    prog='capture_video',
    description='zenoh face recognition example video capture')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--connect', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to connect to.')
parser.add_argument('-l', '--listen', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to listen on.')
parser.add_argument('-i', '--id', type=int, default=random.randint(1, 999),
                    help='The Camera ID.')
parser.add_argument('-w', '--width', type=int, default=500,
                    help='width of the published frames')
parser.add_argument('-q', '--quality', type=int, default=95,
                    help='quality of the published frames (0 - 100)')
parser.add_argument('-d', '--delay', type=float, default=0.05,
                    help='delay between each frame in seconds')
parser.add_argument('-p', '--prefix', type=str, default='demo/facerecog',
                    help='resources prefix')
parser.add_argument('-c', '--config', type=str, metavar='FILE',
                    help='A zenoh configuration file.')

args = parser.parse_args()
conf = zenoh.config_from_file(args.config) if args.config is not None else zenoh.Config()
if args.mode is not None:
    conf.insert_json5(zenoh.config.MODE_KEY, json.dumps(args.mode))
if args.connect is not None:
    conf.insert_json5(zenoh.config.CONNECT_KEY, json.dumps(args.connect))
if args.listen is not None:
    conf.insert_json5(zenoh.config.LISTEN_KEY, json.dumps(args.listen))

jpeg_opts = [int(cv2.IMWRITE_JPEG_QUALITY), args.quality]
cam_id = args.id

print('[INFO] Open zenoh session...')

zenoh.init_logger()
z = zenoh.open(conf)

print('[INFO] Start video stream - Cam #{}'.format(cam_id))
vs = VideoStream(src=0).start()
time.sleep(1.0)

while True:
    raw = vs.read()
    frame = imutils.resize(raw, width=500)

    _, jpeg = cv2.imencode('.jpg', frame, jpeg_opts)

    # print('[DEBUG] Put frame: {}/cams/{}'.format(args.prefix, cam_id))
    z.put('{}/cams/{}'.format(args.prefix, cam_id), jpeg.tobytes())

    time.sleep(args.delay)

vs.stop()
z.close()

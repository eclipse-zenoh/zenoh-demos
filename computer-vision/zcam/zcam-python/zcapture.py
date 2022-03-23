import argparse
from email.policy import default
from imutils.video import VideoStream
import imutils
import time
import cv2
import zenoh
import json

CAMERA_ID                   = 0

parser = argparse.ArgumentParser(
    prog='capture_video',
    description='zenoh face recognition example face detector')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--peer', type=str, metavar='LOCATOR', action='append',
                    help='Peer locators used to initiate the zenoh session.')
parser.add_argument('-l', '--listener', type=str, metavar='LOCATOR', action='append',
                    help='Locators to listen on.')
parser.add_argument('-w', '--width', type=int, default=500,
                    help='width of the published frames')
parser.add_argument('-q', '--quality', type=int, default=95,
                    help='quality of the published frames (0 - 100)')
parser.add_argument('-d', '--delay', type=float, default=0.05,
                    help='delay between each frame in seconds')
parser.add_argument('-k', '--key', type=str, default='/demo/zcam',
                    help='key expression')
parser.add_argument('-c', '--config', type=str, metavar='FILE',
                    help='A zenoh configuration file.')

args = parser.parse_args()

jpeg_opts = [int(cv2.IMWRITE_JPEG_QUALITY), args.quality]

conf = zenoh.config_from_file(args.config) if args.config is not None else zenoh.Config()
if args.mode is not None:
    conf.insert_json5("mode", json.dumps(args.mode))
if args.peer is not None:
    conf.insert_json5("connect/endpoints", json.dumps(args.peer))
if args.listener is not None:
    conf.insert_json5("listeners", json.dumps(args.listener))


print('[INFO] Open zenoh session...')
zenoh.init_logger()
z = zenoh.open(conf)

print('[INFO] Open camera...')
vs = VideoStream(src=CAMERA_ID).start()

time.sleep(1.0)

while True:

    raw = vs.read()
    if raw is not None:
        frame = imutils.resize(raw, width=args.width)
        _, jpeg = cv2.imencode('.jpg', frame, jpeg_opts)
        z.put(args.key, jpeg.tobytes())

    time.sleep(args.delay)

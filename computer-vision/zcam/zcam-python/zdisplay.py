import argparse
import time
import cv2
import zenoh
import numpy as np
import json

parser = argparse.ArgumentParser(
    prog='zdisplay',
    description='zenoh video display example')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--peer', type=str, metavar='LOCATOR', action='append',
                    help='Peer locators used to initiate the zenoh session.')
parser.add_argument('-l', '--listener', type=str, metavar='LOCATOR', action='append',
                    help='Locators to listen on.')
parser.add_argument('-d', '--delay', type=float, default=0.05,
                    help='delay between each frame in seconds')
parser.add_argument('-k', '--key', type=str, default='/rt/*/cams/*',
                    help='key expression')
parser.add_argument('-c', '--config', type=str, metavar='FILE',
                    help='A zenoh configuration file.')

args = parser.parse_args()
conf = zenoh.config_from_file(args.config) if args.config is not None else zenoh.Config()
if args.mode is not None:
    conf.insert_json5("mode", json.dumps(args.mode))
if args.peer is not None:
    conf.insert_json5("connect/endpoints", json.dumps(args.peer))
if args.listener is not None:
    conf.insert_json5("listeners", json.dumps(args.listener))

cams = {}

def frames_listener(sample):
    npImage = np.frombuffer(bytes(sample.value.payload), dtype=np.uint8)
    matImage = cv2.imdecode(npImage, 1)

    cams[sample.key_expr] = matImage


print('[INFO] Open zenoh session...')
zenoh.init_logger()
z = zenoh.open(conf)

sub = z.subscribe(args.key, frames_listener)

while True:
    for cam in list(cams):
        cv2.imshow(str(cam), cams[cam])

    key = cv2.waitKey(1) & 0xFF
    time.sleep(args.delay)

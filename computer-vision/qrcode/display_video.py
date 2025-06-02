import argparse
import time
import cv2
import json
import random
import zenoh
import numpy as np
import json

parser = argparse.ArgumentParser(
    prog='display_video',
    description='zenoh face recognition example display')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--connect', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to connect to.')
parser.add_argument('-l', '--listen', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to listen on.')
parser.add_argument('-d', '--delay', type=float, default=0.05,
                    help='delay between each frame in seconds')
parser.add_argument('-p', '--prefix', type=str, default='demo/qrcode',
                    help='resources prefix')
parser.add_argument('-c', '--config', type=str, metavar='FILE',
                    help='A zenoh configuration file.')

args = parser.parse_args()
conf = zenoh.Config.from_file(args.config) if args.config is not None else zenoh.Config()
if args.mode is not None:
    conf.insert_json5('mode', json.dumps(args.mode))
if args.connect is not None:
    conf.insert_json5('connect/endpoints', json.dumps(args.connect))
if args.listen is not None:
    conf.insert_json5('listen/endpoints', json.dumps(args.listen))

cams = {}

def frames_listener(sample):
    # print('[DEBUG] Received frame: {}'.format(sample.key_expr))
    chunks = str(sample.key_expr).split('/')
    cam = chunks[-1]

    if cam not in cams:
        cams[cam] = {}
    cams[cam]['img'] = bytes(sample.payload)

def codes_listener(sample):
    # print('[DEBUG] Received code: {} => {}'.format(sample.key_expr, sample.payload.decode("utf-8")))
    chunks = str(sample.key_expr).split('/')
    cam = chunks[-2]
    code = int(chunks[-1])

    if cam not in cams:
        cams[cam] = {}
    if 'codes' not in cams[cam]:
        cams[cam]['codes'] = {}
    if code not in cams[cam]['codes']:
        cams[cam]['codes'][code] = {}

    cams[cam]['codes'][code] = json.loads(sample.payload.to_string())
    cams[cam]['codes'][code]['time'] = time.time()

print('[INFO] Open zenoh session...')

zenoh.init_log_from_env_or("error")
z = zenoh.open(conf)

sub = z.declare_subscriber(args.prefix + '/cams/*', frames_listener)
sub2 = z.declare_subscriber(args.prefix + '/codes/*/*', codes_listener)

while True:
    now = time.time()
    for cam in list(cams):
        if 'img' in cams[cam]:
            npImage = np.frombuffer(cams[cam]['img'], dtype=np.uint8)
            matImage = cv2.imdecode(npImage, 1)
            if 'codes' in cams[cam]:
                for code in cams[cam]['codes']:
                    if cams[cam]['codes'][code]['time'] > now - 0.2:
                        cv2.putText(matImage, cams[cam]['codes'][code]['info'],
                            np.array(cams[cam]['codes'][code]['box'][0]).astype(int),
                            cv2.FONT_HERSHEY_SIMPLEX,
                            0.6,
                            (255, 0, 0),
                            2)
                        cv2.polylines(matImage, [np.array(cams[cam]['codes'][code]['box']).astype(int)], True, (255, 0, 0), 2)

            cv2.imshow('Cam #' + cam, matImage)

    key = cv2.waitKey(1) & 0xFF
    time.sleep(args.delay)

vs.stop()
z.close()

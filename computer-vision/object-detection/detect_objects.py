import argparse
import time
import cv2
import json
import random
import zenoh
import numpy as np
from ultralytics import YOLO

parser = argparse.ArgumentParser(
    prog='detect',
    description='zenoh object detection example')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--connect', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to connect to.')
parser.add_argument('-l', '--listen', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to listen on.')
parser.add_argument('-d', '--delay', type=float, default=0.05,
                    help='delay between each frame in seconds')
parser.add_argument('-s', '--model-size', type=str, default="s",
                    help='The size of the model to use. Can be \'n\', \'s\', \'m\', \'l\' or \'x\'.')
parser.add_argument('-p', '--prefix', type=str, default='demo/obj-detect',
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

qcd = cv2.QRCodeDetector()
cams = {}
model = YOLO("yolo26" + args.model_size + ".pt")

def frames_listener(sample):
    # print('[DEBUG] Received frame: {}'.format(sample.key_expr))
    chunks = str(sample.key_expr).split('/')
    cam = chunks[-1]

    cams[cam] = bytes(sample.payload)


print('[INFO] Open zenoh session...')

zenoh.init_log_from_env_or("error")
z = zenoh.open(conf)

print('[INFO] Start detection')
sub = z.declare_subscriber(args.prefix + '/cams/*', frames_listener)

while True:
    for cam in list(cams):
        npImage = np.frombuffer(cams[cam], dtype=np.uint8)
        matImage = cv2.imdecode(npImage, 1)


        results = model.predict(source=matImage, show_boxes=True, verbose=False)
        i = 0
        for result in results:
            for box in result.boxes:
                for data in box.data:
                    box = [[int(data[0]), int(data[1])], 
                           [int(data[2]), int(data[1])], 
                           [int(data[2]), int(data[3])], 
                           [int(data[0]), int(data[3])]]
                    z.put('{}/objects/{}/{}'.format(args.prefix, cam, i),
                        json.dumps({'info': result.names[int(data[5])], 'box': box}))
                    i += 1
                

    time.sleep(args.delay)

vs.stop()
z.close()

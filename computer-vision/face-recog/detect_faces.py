import argparse
import imutils
import time
import io
import cv2
import random
import zenoh
import binascii
import numpy as np

parser = argparse.ArgumentParser(
    prog='detect_faces',
    description='zenoh face recognition example face detector')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--peer', type=str, metavar='LOCATOR', action='append',
                    help='Peer locators used to initiate the zenoh session.')
parser.add_argument('-l', '--listener', type=str, metavar='LOCATOR', action='append',
                    help='Locators to listen on.')
parser.add_argument('-i', '--id', type=int, default=random.randint(1, 999),
                    help='The Camera ID.')
parser.add_argument('-w', '--width', type=int, default=200,
                    help='width of the published faces')
parser.add_argument('-q', '--quality', type=int, default=95,
                    help='quality of the published faces (0 - 100)')
parser.add_argument('-a', '--cascade', type=str,
                    default='haarcascade_frontalface_default.xml',
                    help='path to the face cascade file')
parser.add_argument('-d', '--delay', type=float, default=0.05,
                    help='delay between each frame in seconds')
parser.add_argument('-p', '--prefix', type=str, default='/demo/facerecog',
                    help='resources prefix')
parser.add_argument('-c', '--config', type=str, metavar='FILE',
                    help='A zenoh configuration file.')

args = parser.parse_args()
conf = zenoh.config_from_file(args.config) if args.config is not None else zenoh.Config()
if args.mode is not None:
    conf.insert_json5("mode", json.dumps(args.mode))
if args.peer is not None:
    conf.insert_json5("peers", json.dumps(args.peer))
if args.listener is not None:
    conf.insert_json5("listeners", json.dumps(args.listener))

jpeg_opts = [int(cv2.IMWRITE_JPEG_QUALITY), args.quality]
cams = {}


def frames_listener(sample):
    # print('[DEBUG] Received frame: {}'.format(sample.key_expr))
    chunks = str(sample.key_expr).split('/')
    cam = chunks[-1]

    cams[cam] = bytes(sample.value.payload)


print('[INFO] Open zenoh session...')

zenoh.init_logger()
z = zenoh.open(conf)

detector = cv2.CascadeClassifier(args.cascade)

print('[INFO] Start detection')
sub = z.subscribe(args.prefix + '/cams/*', frames_listener)

while True:
    for cam in list(cams):
        npImage = np.load(io.BytesIO(cams[cam]), allow_pickle=True)
        matImage = cv2.imdecode(npImage, 1)

        gray = cv2.cvtColor(matImage, cv2.COLOR_BGR2GRAY)

        rects = detector.detectMultiScale(gray, scaleFactor=1.1,
                                          minNeighbors=5, minSize=(30, 30),
                                          flags=cv2.CASCADE_SCALE_IMAGE)
        boxes = [(y, x + w, y + h, x) for (x, y, w, h) in rects]

        faces = zip(range(len(boxes)), sorted(boxes))

        for (i, (top, right, bottom, left)) in faces:
            face = matImage[int(top):int(bottom),
                            int(left):int(right)]
            face = imutils.resize(face, width=args.width)
            _, jpeg = cv2.imencode('.jpg', face, jpeg_opts)
            buf = io.BytesIO()
            np.save(buf, jpeg, allow_pickle=True)

            # print('[DEBUG] Put detected face: {}/faces/{}/{}'.format(args.prefix, cam, i))
            z.put('{}/faces/{}/{}'.format(args.prefix, cam, i), buf.getvalue())

    time.sleep(args.delay)

vs.stop()
z.close()

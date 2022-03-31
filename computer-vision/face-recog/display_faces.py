import zenoh
import argparse
import imutils
import io
import cv2
import time
import numpy as np

parser = argparse.ArgumentParser(
    prog='display_faces',
    description='zenoh face recognition example display')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--peer', type=str, metavar='LOCATOR', action='append',
                    help='Peer locators used to initiate the zenoh session.')
parser.add_argument('-l', '--listener', type=str, metavar='LOCATOR', action='append',
                    help='Locators to listen on.')
parser.add_argument('-p', '--prefix', type=str, default='/demo/facerecog',
                    help='resources prefix')
parser.add_argument('-d', '--delay', type=float, default=0.05,
                    help='delay between each refresh')
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

cams = {}


def faces_listener(sample):
    # print('[DEBUG] Received face: '+sample.key_expr)
    chunks = str(sample.key_expr).split('/')
    cam = chunks[-2]
    face = int(chunks[-1])

    if cam not in cams:
        cams[cam] = {}
    if face not in cams[cam]:
        cams[cam][face] = {'img': b'', 'name': '', 'time': 0}

    cams[cam][face]['img'] = bytes(sample.payload)
    cams[cam][face]['time'] = time.time()


def names_listener(sample):
    # print('[DEBUG] Received name: {} {} => {}', sample.key_expr, sample.payload)
    chunks = str(sample.key_expr).split('/')
    cam = chunks[-3]
    face = int(chunks[-2])

    if cam not in cams:
        cams[cam] = {}
    if face not in cams[cam]:
        cams[cam][face] = {'img': b'', 'name': '', 'time': 0}

    cams[cam][face]['name'] = sample.payload.decode("utf-8")


print('[INFO] Open zenoh session...')
zenoh.init_logger()
z = zenoh.open(conf)
sub1 = z.subscribe(args.prefix + '/faces/*/*', faces_listener)
sub2 = z.subscribe(args.prefix + '/faces/*/*/name', names_listener)

for data in z.get(args.prefix + '/faces/*/*/name'):
    names_listener(data)

print('[INFO] Display detected faces ...')

while True:
    now = time.time()

    for cam in list(cams):
        faces = cams[cam]
        vbuf = np.zeros((250, 1000, 3), np.uint8)
        for face in list(faces):
            if faces[face]['time'] > now - 0.2:
                npImage = np.frombuffer(faces[face]['img'], dtype=np.uint8)
                matImage = cv2.imdecode(npImage, 1)
                resImage = imutils.resize(matImage, width=200)
                h, w, _ = resImage.shape
                vbuf[40:40+h, 200*face:200*face+w] = resImage

                name = faces[face]['name']
                color = (0, 0, 255) if name == 'Unknown' else (0, 255, 0)
                cv2.putText(vbuf,
                            name,
                            (200*face + 2, 18),
                            cv2.FONT_HERSHEY_SIMPLEX,
                            0.75,
                            color,
                            2)

        cv2.imshow('Cam #' + cam, vbuf)

    time.sleep(args.delay)

    key = cv2.waitKey(1) & 0xFF
    if key == ord('q'):
        break

cv2.destroyAllWindows()
sub1.close()
sub2.close()
z.close()

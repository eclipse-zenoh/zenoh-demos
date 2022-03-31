import argparse
import time
import io
import ast
import cv2
import numpy as np
import face_recognition
import zenoh

parser = argparse.ArgumentParser(
    prog='recognize_faces',
    description='zenoh face recognition example')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--peer', type=str, metavar='LOCATOR', action='append',
                    help='Peer locators used to initiate the zenoh session.')
parser.add_argument('-l', '--listener', type=str, metavar='LOCATOR', action='append',
                    help='Locators to listen on.')
parser.add_argument('-p', '--prefix', type=str, default='/demo/facerecog',
                    help='The resources prefix')
parser.add_argument('-d', '--delay', type=float, default=0.2,
                    help='delay between each recognition')
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

data = {}
data['encodings'] = []
data['names'] = []
cams = {}


def add_face_to_data(fdata, key, value):
    chunks = key.split('/')
    name = chunks[-2]
    num = chunks[-1]
    print('[INFO] Add face to recognize: {}/{}'.format(name, num))
    fdata['names'].append(name)
    a = ast.literal_eval(value)
    fdata['encodings'].append(a)


def update_face_data(sample):
    if sample.kind == zenoh.SampleKind.PUT:
        add_face_to_data(data, str(sample.key_expr), sample.payload.decode("utf-8"))


def faces_listener(sample):
    # print('[DEBUG] Received face to recognize: {}'.format(sample.key_expr))
    chunks = str(sample.key_expr).split('/')
    cam = chunks[-2]
    face = int(chunks[-1])

    if cam not in cams:
        cams[cam] = {}

    cams[cam][face] = bytes(sample.payload)


print('[INFO] Open zenoh session...')
zenoh.init_logger()
z = zenoh.open(conf)
time.sleep(0.5)

print('[INFO] Retrieve faces vectors...')
for vector in z.get(args.prefix + '/vectors/**'):
    add_face_to_data(data, str(vector.data.key_expr), vector.data.payload.decode("utf-8"))

print('[INFO] Start recognition...')
sub1 = z.subscribe(args.prefix + '/vectors/**', update_face_data)
sub2 = z.subscribe(args.prefix + '/faces/*/*', faces_listener)

while True:
    for cam in list(cams):
        faces = cams[cam]
        for face in list(faces):
            npImage = np.frombuffer(faces[face], dtype=np.uint8)
            matImage = cv2.imdecode(npImage, 1)
            rgb = cv2.cvtColor(matImage, cv2.COLOR_BGR2RGB)

            encodings = face_recognition.face_encodings(rgb)

            name = 'Unknown'
            if len(encodings) > 0:
                matches = face_recognition.compare_faces(data['encodings'],
                                                         encodings[0])
                if True in matches:
                    matchedIdxs = [i for (i, b) in enumerate(matches) if b]
                    counts = {}
                    for i in matchedIdxs:
                        name = data['names'][i]
                        counts[name] = counts.get(name, 0) + 1
                    name = max(counts, key=counts.get)

            path = args.prefix + '/faces/' + cam + '/' + str(face) + '/name'
            # print('[DEBUG] Name for {} : {}'.format(path, name))
            z.put(path, name)

    time.sleep(args.delay)

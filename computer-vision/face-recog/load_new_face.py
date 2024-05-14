import argparse
import time
import json
import cv2
import imutils
from imutils.video import VideoStream
import face_recognition
import zenoh

parser = argparse.ArgumentParser(
    prog='detect_faces',
    description='zenoh face recognition example face loader')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--connect', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to connect to.')
parser.add_argument('-l', '--listen', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to listen on.')
parser.add_argument('-n', '--name', required=True,
                    help='The name of the person')
parser.add_argument('-p', '--prefix', type=str, default='demo/facerecog',
                    help='The resources prefix')
parser.add_argument('-a', '--cascade', type=str,
                    default='haarcascade_frontalface_default.xml',
                    help='path to the face cascade file')
parser.add_argument('-d', '--detection-method', type=str, default='cnn',
                    help='face detection model to use: either `hog` or `cnn`')
parser.add_argument('-c', '--config', type=str, metavar='FILE',
                    help='A zenoh configuration file.')

args = parser.parse_args()
conf = zenoh.Config.from_file(args.config) if args.config is not None else zenoh.Config()
if args.mode is not None:
    conf.insert_json5(zenoh.config.MODE_KEY, json.dumps(args.mode))
if args.connect is not None:
    conf.insert_json5(zenoh.config.CONNECT_KEY, json.dumps(args.connect))
if args.listen is not None:
    conf.insert_json5(zenoh.config.LISTEN_KEY, json.dumps(args.listen))

detector = cv2.CascadeClassifier(args.cascade)

print('[INFO] Open zenoh session...')
zenoh.init_logger()
z = zenoh.open(conf)

vs = VideoStream(src=0).start()

while True:
    frame = vs.read()
    frame = imutils.resize(frame, width=500)

    gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
    rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)

    rects = detector.detectMultiScale(gray, scaleFactor=1.1,
                                      minNeighbors=5, minSize=(30, 30),
                                      flags=cv2.CASCADE_SCALE_IMAGE)

    if len(rects) > 0:
        cv2.rectangle(frame,
                      (rects[0][0], rects[0][1]),
                      (rects[0][0]+rects[0][2], rects[0][1]+rects[0][3]),
                      (0, 255, 0), 2)

    cv2.putText(frame, 'Press <space> to take picture <q> to quit.', (10, 250),
                cv2.FONT_HERSHEY_SIMPLEX,
                0.65, (0, 255, 0), 2)
    cv2.imshow('Register new face vector', frame)

    key = cv2.waitKey(1) & 0xFF
    if key == ord(' '):
        if len(rects) > 0:
            face = frame[rects[0][1]:rects[0][1]+rects[0][3],
                         rects[0][0]:rects[0][0]+rects[0][2]]
            box = [(rects[0][1], rects[0][0]+rects[0][2],
                    rects[0][1]+rects[0][3], rects[0][0])]
            encoding = face_recognition.face_encodings(rgb, box)[0]
            elist = encoding.tolist()

            faces = z.get(args.prefix + '/vectors/**', zenoh.ListCollector())
            counter = 0
            for face in faces():
                chunks = str(face.ok.key_expr).split('/')
                name = chunks[-2]
                if name == args.name:
                    if counter <= int(chunks[-1]):
                        counter = int(chunks[-1]) + 1

            uri = '{}/vectors/{}/{}'.format(
                args.prefix, args.name, str(counter))
            print('> Inserting face vector {}'.format(uri))
            z.put(uri, json.dumps(elist))

    time.sleep(0.05)

    if key == ord('q'):
        exit(0)

z.close()

import argparse
import imutils
import time
import cv2
import zenoh
import json

CAMERA_ID                   = 0

parser = argparse.ArgumentParser(
    prog='zcapture',
    description='zenoh video capture example')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--connect', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to connect to.')
parser.add_argument('-l', '--listen', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to listen on.')
parser.add_argument('-a', '--camera', type=str, default='default', choices=['default', 'picameraV1', 'picameraV2'],
                    help='The type of camera to use.')
parser.add_argument('-w', '--width', type=int, default=500,
                    help='width of the published frames')
parser.add_argument('-q', '--quality', type=int, default=95,
                    help='quality of the published frames (0 - 100)')
parser.add_argument('-d', '--delay', type=float, default=0.05,
                    help='delay between each frame in seconds')
parser.add_argument('-k', '--key', type=str, default='demo/zcam',
                    help='key expression')
parser.add_argument('-c', '--config', type=str, metavar='FILE',
                    help='A zenoh configuration file.')

args = parser.parse_args()

jpeg_opts = [int(cv2.IMWRITE_JPEG_QUALITY), args.quality]
picamera = args.camera.startswith('picamera')

conf = zenoh.Config.from_file(args.config) if args.config is not None else zenoh.Config()
if args.mode is not None:
    conf.insert_json5('mode', json.dumps(args.mode))
if args.connect is not None:
    conf.insert_json5('connect/endpoints', json.dumps(args.connect))
if args.listen is not None:
    conf.insert_json5('listen/endpoints', json.dumps(args.listen))


print('[INFO] Open zenoh session...')
zenoh.init_log_from_env_or("error")
z = zenoh.open(conf)

print('[INFO] Open camera...')
if args.camera == 'picameraV1':
    import picamera2
    vs = picamera2.Picamera2()
    vs.configure(vs.create_preview_configuration({'format': 'XRGB8888', 'size': (1296, 972)}))
    vs.start()
elif args.camera == 'picameraV2':
    import picamera2
    vs = picamera2.Picamera2()
    vs.configure(vs.create_preview_configuration({'format': 'XRGB8888', 'size': (1640, 1232)}))
    vs.start()
else:
    from imutils.video import VideoStream
    vs = VideoStream(src=CAMERA_ID).start()

time.sleep(1.0)

while True:
    if picamera:
        raw = vs.capture_array()
    else:
        raw = vs.read()
    if raw is not None:
        frame = imutils.resize(raw, width=args.width)
        _, jpeg = cv2.imencode('.jpg', frame, jpeg_opts)
        z.put(args.key, jpeg.tobytes())

    time.sleep(args.delay)

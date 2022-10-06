import argparse
from email.policy import default
from imutils.video import VideoStream
import imutils
import time
import io
import cv2
import random
import zenoh
import binascii
import numpy as np
import json
import subprocess
from servo import *
from pycdr import cdr
from pycdr.types import int8, int32, uint32, float64

@cdr
class Vector3:
    x: float64
    y: float64
    z: float64

@cdr
class Twist:
    linear: Vector3
    angular: Vector3

DEVICENAME                  = '/dev/ttyACM0'
PROTOCOL_VERSION            = 2.0
BAUDRATE                    = 115200
MOTOR_ID                    = 200
CAMERA_ID                   = 0

def getBSSID():
    result=subprocess.run(['iwconfig'], capture_output=True)
    bssid=next(l for l in result.stdout.decode("utf-8").splitlines() if "Access Point: " in l).split("Access Point: ",1)[1].split(" ", 1)[0]
    if not bssid:
        return None
    else:
        return bssid

def getEndpoint(mapping, bssid):
    if bssid in mapping:
        return mapping[bssid]
    elif "default" in mapping:
        return mapping["default"]

parser = argparse.ArgumentParser(
    prog='zturtle',
    description='zenoh turtlebot3 example')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--endpoints', type=str, required=True,
                    help='A BSSID/endpoint mapping json file.')
parser.add_argument('-l', '--listen', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to listen on.')
parser.add_argument('-w', '--width', type=int, default=500,
                    help='width of the published frames')
parser.add_argument('-q', '--quality', type=int, default=95,
                    help='quality of the published frames (0 - 100)')
parser.add_argument('-d', '--delay', type=float, default=0.05,
                    help='delay between each iteration in seconds')
parser.add_argument('-p', '--prefix', type=str, default='rt/turtle1',
                    help='resources prefix')
parser.add_argument('-c', '--config', type=str, metavar='FILE',
                    help='A zenoh configuration file.')

args = parser.parse_args()

count = 0
cmd = Twist(Vector3(0.0, 0.0, 0.0), Vector3(0.0, 0.0, 0.0))
jpeg_opts = [int(cv2.IMWRITE_JPEG_QUALITY), args.quality]
bssid = getBSSID()
mapping = json.loads(open(args.endpoints, "r").read())

conf = zenoh.config_from_file(args.config) if args.config is not None else zenoh.Config()
conf.insert_json5(zenoh.config.CONNECT_KEY, getEndpoint(mapping, bssid))
if args.mode is not None:
    conf.insert_json5(zenoh.config.MODE_KEY, json.dumps(args.mode))
if args.listen is not None:
    conf.insert_json5(zenoh.config.LISTEN_KEY, json.dumps(args.listen))


print('[INFO] Open zenoh session...')
zenoh.init_logger()
z = zenoh.open(conf)

heartbeat_pub = z.declare_publisher('{}/heartbeat'.format(args.prefix))

def listener(sample):
    global cmd
    cmd = Twist.deserialize(sample.value.payload)

print('[INFO] Connect to motor...')
servo = Servo(DEVICENAME, PROTOCOL_VERSION, BAUDRATE, MOTOR_ID)
if servo is None:
    print('[WARN] Unable to connect to motor.')
else:
    servo.write1ByteTxRx(IMU_RE_CALIBRATION, 1)
    sub = z.declare_subscriber('{}/cmd_vel'.format(args.prefix), listener)

print('[INFO] Open camera...')
vs = VideoStream(src=CAMERA_ID).start()
cam_pub = z.declare_publisher('{}/cams/0'.format(args.prefix))

time.sleep(3.0)

print('[INFO] Running!')
while True:
    if servo is not None:
        servo.write1ByteTxRx(HEARTBEAT, count)
        servo.write4ByteTxRx(CMD_VELOCITY_LINEAR_X, int(cmd.linear.x))
        servo.write4ByteTxRx(CMD_VELOCITY_LINEAR_Y, int(cmd.linear.y))
        servo.write4ByteTxRx(CMD_VELOCITY_LINEAR_Z, int(cmd.linear.z))
        servo.write4ByteTxRx(CMD_VELOCITY_ANGULAR_X, int(cmd.angular.x))
        servo.write4ByteTxRx(CMD_VELOCITY_ANGULAR_Y, int(cmd.angular.y))
        servo.write4ByteTxRx(CMD_VELOCITY_ANGULAR_Z, int(cmd.angular.z))
    cmd = Twist(Vector3(0.0, 0.0, 0.0), Vector3(0.0, 0.0, 0.0))

    heartbeat_pub.put(count)

    raw = vs.read()
    if raw is not None:
        frame = imutils.resize(raw, width=args.width)
        _, jpeg = cv2.imencode('.jpg', frame, jpeg_opts)
        cam_pub.put(jpeg.tobytes())

    new_bssid = getBSSID()
    if new_bssid != bssid:
        print("[info] New access point detected")
        bssid = new_bssid
        peer = getEndpoint(mapping, bssid)
        if peer:
            z.config().insert_json5(zenoh.config.CONNECT_KEY, peer)

    count += 1
    if count > 255:
        count = 0

    time.sleep(args.delay)

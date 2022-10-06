import argparse
import time
import io
import zenoh
import json
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

parser = argparse.ArgumentParser(
    prog='drive_motors',
    description='zenoh drive_motors example')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--connect', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to connect to.')
parser.add_argument('-l', '--listen', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to listen on.')
parser.add_argument('-d', '--delay', type=float, default=0.1,
                    help='delay between each iteration in seconds')
parser.add_argument('-p', '--prefix', type=str, default='rt/turtle1',
                    help='resources prefix')
parser.add_argument('-c', '--config', type=str, metavar='FILE',
                    help='A zenoh configuration file.')

args = parser.parse_args()

count = 0
cmd = Twist(Vector3(0.0, 0.0, 0.0), Vector3(0.0, 0.0, 0.0))

conf = zenoh.config_from_file(args.config) if args.config is not None else zenoh.Config()
if args.connect is not None:
    conf.insert_json5(zenoh.config.CONNECT_KEY, json.dumps(args.connect))
if args.mode is not None:
    conf.insert_json5(zenoh.config.MODE_KEY, json.dumps(args.mode))
if args.listen is not None:
    conf.insert_json5(zenoh.config.LISTEN_KEY, json.dumps(args.listen))


print('[INFO] Open zenoh session...')
zenoh.init_logger()
z = zenoh.open(conf)

publ = z.declare_publisher('{}/heartbeat'.format(args.prefix))

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

    publ.put(count)

    count += 1
    if count > 255:
        count = 0

    time.sleep(args.delay)

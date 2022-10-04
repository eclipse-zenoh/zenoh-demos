import argparse
import json
from turtle import stamp
import zenoh
import json
import cmath
import numpy as np
from matplotlib.animation import FuncAnimation
from matplotlib import pyplot as plt
from pycdr import cdr
from pycdr.types import int8, int32, uint16, uint32, float32, sequence, array

@cdr
class Time:
    sec: uint32
    nsec: uint32

@cdr
class Header:
    stamp: Time
    frame_id: str

@cdr
class LaserScan:
    stamp_sec: uint32
    stamp_nsec: uint32
    frame_id: str
    angle_min: float32
    angle_max: float32
    angle_increment: float32
    time_increment: float32
    scan_time: float32
    range_min: float32
    range_max: float32
    ranges: array[float32, 360]
    intensities: array[float32, 360]

parser = argparse.ArgumentParser(
    prog='zlidar-plot',
    description='zenoh turtlebot3 lidar plot display')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--connect', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to connect to.')
parser.add_argument('-l', '--listen', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to listen on.')
parser.add_argument('-k', '--key', type=str, default='rt/turtle1/scan',
                    help='The key expression to subscribe for LaserReadings.')
parser.add_argument('--style', type=str, default='.',
                    help='The plotting style.')
parser.add_argument('--intensity-treshold', type=float, default=250.0,
                    help='The intensity treshold.')
parser.add_argument('-c', '--config', type=str, metavar='FILE',
                    help='A zenoh configuration file.')

args = parser.parse_args()
conf = zenoh.config_from_file(args.config) if args.config is not None else zenoh.Config()
if args.mode is not None:
    conf.insert_json5(zenoh.config.MODE_KEY, json.dumps(args.mode))
if args.connect is not None:
    conf.insert_json5(zenoh.config.CONNECT_KEY, json.dumps(args.connect))
if args.listen is not None:
    conf.insert_json5(zenoh.config.LISTEN_KEY, json.dumps(args.listen))

fig, ax = plt.subplots()
angles = list(map(lambda i: (2*cmath.pi*i/360)*1j-cmath.pi/2j, range(360)))
line = ax.plot([], [], args.style)[0]
center = ax.plot([0], [0], 'o')[0]
ax.set_xlim(-4, 4)
ax.set_ylim(-4, 4)


def lidar_listener(sample):
    # print('[DEBUG] Received frame: {}'.format(sample.key_expr))
    scan = LaserScan.deserialize(sample.payload)

    complexes = []
    for (angle, distance, intensity) in list(zip(angles, scan.ranges, scan.intensities)):
        complexes.append(distance * cmath.exp(angle) if intensity >= args.intensity_treshold else np.nan)
    X = [i.real for i in complexes]
    Y = [i.imag for i in complexes]
    global line
    line.set_data(X, Y)

print("[INFO] Openning zenoh session...")
zenoh.init_logger()
z = zenoh.open(conf)

print("[INFO] Creating Subscriber on '{}'...".format(args.key))
sub = z.declare_subscriber(args.key, lidar_listener)

ani = FuncAnimation(fig, lambda _: None)
plt.show()

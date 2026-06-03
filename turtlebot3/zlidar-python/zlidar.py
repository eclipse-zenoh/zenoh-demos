import argparse
import json
import math
from dataclasses import dataclass
import serial
from pycdr2 import IdlStruct
from pycdr2.types import uint32, float32
from typing import List
import zenoh

@dataclass
class Time(IdlStruct, typename="Time"):
    sec: uint32
    nsec: uint32

@dataclass
class Header(IdlStruct, typename="Header"):
    stamp: Time
    frame_id: str

@dataclass
class LaserScan(IdlStruct, typename="LaserScan"):
    header: Header
    angle_min: float32
    angle_max: float32
    angle_increment: float32
    time_increment: float32
    scan_time: float32
    range_min: float32
    range_max: float32
    ranges: List[float32]
    intensities: List[float32]

parser = argparse.ArgumentParser(
    prog='zlidar',
    description='zenoh turtlebot3 lidar capture')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--connect', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to connect to.')
parser.add_argument('-l', '--listen', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to listen on.')
parser.add_argument('-k', '--key', type=str, default='rt/turtle1/lidar',
                    help='The key expression to publish LaserReadings to.')
parser.add_argument('-c', '--config', type=str, metavar='FILE',
                    help='A zenoh configuration file.')
parser.add_argument('-p', '--port', type=str, default='/dev/ttyUSB0',
                    help='The serial port to read LaserReadings from.')
parser.add_argument('-b', '--baud-rate', type=int, default='115200',
                    help='The baud rate.')

args = parser.parse_args()
conf = zenoh.Config.from_file(args.config) if args.config is not None else zenoh.Config()
if args.mode is not None:
    conf.insert_json5('mode', json.dumps(args.mode))
if args.connect is not None:
    conf.insert_json5('connect/endpoints', json.dumps(args.connect))
if args.listen is not None:
    conf.insert_json5('listen/endpoints', json.dumps(args.listen))

ser = serial.Serial (args.port, baudrate=args.baud_rate)

print("[INFO] Openning zenoh session...")
zenoh.init_log_from_env_or("error")
z = zenoh.open(conf)

p = z.declare_publisher(args.key)

print("[INFO] Publishing laser scans on", args.key, "...")
ranges = []
intensities = []
angle_min = -1.0
angle_max = -1.0

while 1:
    b = ser.read(47)
    read_angle_min = int.from_bytes(b[4:6], 'little') * 0.00017453293 # pi/(180 * 100)
    read_angle_max = int.from_bytes(b[42:44], 'little') * 0.00017453293 # pi/(180 * 100)
    if angle_min == -1.0:
        angle_min = read_angle_min
    if read_angle_max < read_angle_min:
        read_angle_max = read_angle_max + math.pi * 2
    
    for i in range(12):
        angle = (read_angle_min + i * ((read_angle_max - read_angle_min) / 11)) % (math.pi * 2)
        range_ = int.from_bytes(b[i*3+6:i*3+8], 'little') * 0.001
        intensity =  b[i*3+8] * 1.0
        if angle > angle_max:
            angle_max = angle
        else:
            stamp = Time(0, 0) # TODO
            header = Header(stamp, "laser")
            interval = angle_max - angle_min
            if len(ranges) > 1:
                interval = (angle_max - angle_min) / (len(ranges) - 1)
            scan = LaserScan(
                header,
                angle_min,
                angle_max,
                interval,
                0.0, # TODO
                0.0, # TODO
                0.16,
                8.0,
                ranges,
                intensities)
            p.put(scan.serialize())

            ranges = []
            intensities = []
            angle_min = angle
            angle_max = angle
        ranges.append(range_)
        intensities.append(intensity)

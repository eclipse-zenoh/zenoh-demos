import argparse
import json
import math
import time
from dataclasses import dataclass
import serial
from pycdr2 import IdlStruct
from pycdr2.types import uint32, float32
from typing import List
import zenoh

# LD-series lidar serial frame format constants
FRAME_HEADER = 0x54       # Start-of-frame marker byte
FRAME_VERLEN = 0x2C       # Version/length byte (12 measurement points per frame)
FRAME_DATA_LEN = 45       # Bytes following the header+verlen (data + CRC)
POINTS_PER_FRAME = 12     # Measurement points contained in each frame
DEG100_TO_RAD = 0.00017453293  # pi / (180 * 100): converts hundredths of a degree to radians
TWO_PI = math.pi * 2

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
parser.add_argument('-b', '--baud-rate', type=int, default=115200,
                    help='The baud rate.')

args = parser.parse_args()
conf = zenoh.Config.from_file(args.config) if args.config is not None else zenoh.Config()
if args.mode is not None:
    conf.insert_json5('mode', json.dumps(args.mode))
if args.connect is not None:
    conf.insert_json5('connect/endpoints', json.dumps(args.connect))
if args.listen is not None:
    conf.insert_json5('listen/endpoints', json.dumps(args.listen))

# A read timeout keeps read() from blocking forever if the device stalls.
ser = serial.Serial(args.port, baudrate=args.baud_rate, timeout=1.0)

print("[INFO] Opening zenoh session...")
zenoh.init_log_from_env_or("error")
z = zenoh.open(conf)

p = z.declare_publisher(args.key)


def read_frame(ser):
    """Read one lidar frame, resynchronizing on the header byte.

    Returns the FRAME_DATA_LEN-byte payload (measurement data + CRC), or None
    if the read timed out before a complete frame was available.
    """
    while True:
        b = ser.read(1)
        if not b:
            return None  # timeout
        if b[0] != FRAME_HEADER:
            continue  # not a frame start, keep scanning
        b = ser.read(1)
        if not b or b[0] != FRAME_VERLEN:
            continue  # bad length byte, resync from next header
        payload = ser.read(FRAME_DATA_LEN)
        if len(payload) == FRAME_DATA_LEN:
            return payload
        # short read (timeout): resync


print("[INFO] Publishing laser scans on", args.key, "...")
ranges = []
intensities = []
angle_min = -1.0
angle_max = -1.0

try:
    while True:
        bs = read_frame(ser)
        if bs is None:
            continue  # timed out waiting for a frame, try again
        read_angle_min = int.from_bytes(bs[2:4], 'little') * DEG100_TO_RAD
        read_angle_max = int.from_bytes(bs[40:42], 'little') * DEG100_TO_RAD
        if angle_min == -1.0:
            angle_min = read_angle_min
        if read_angle_max < read_angle_min:
            read_angle_max = read_angle_max + TWO_PI

        for i in range(POINTS_PER_FRAME):
            angle = (read_angle_min + i * ((read_angle_max - read_angle_min) / (POINTS_PER_FRAME - 1))) % TWO_PI
            range_ = int.from_bytes(bs[i*3+4:i*3+6], 'little') * 0.001
            intensity = bs[i*3+6] * 1.0
            if angle > angle_max:
                angle_max = angle
            else:
                now = time.time()
                stamp = Time(int(now), int((now % 1) * 1e9))
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
except KeyboardInterrupt:
    print("[INFO] Shutting down...")
finally:
    p.undeclare()
    z.close()
    ser.close()

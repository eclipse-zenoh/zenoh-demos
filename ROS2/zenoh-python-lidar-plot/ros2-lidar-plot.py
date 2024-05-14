#
# Copyright (c) 2022 ZettaScale Technology
#
# This program and the accompanying materials are made available under the
# terms of the Eclipse Public License 2.0 which is available at
# http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
# which is available at https://www.apache.org/licenses/LICENSE-2.0.
#
# SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
#
# Contributors:
#   The Zenoh Team, <zenoh@zettascale.tech>
#   Alexandre Humblot, <alexandre.humblot@student-cs.fr>

import argparse
from dataclasses import dataclass
import json
import zenoh
import cmath
import numpy as np
from matplotlib.animation import FuncAnimation
from matplotlib.patches import Polygon
from matplotlib import pyplot as plt
from pycdr2 import IdlStruct
from pycdr2.types import uint32, float32
from typing import List

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
    prog='zlidar-plot',
    description='zenoh turtlebot3 lidar plot display')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--connect', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to connect to.')
parser.add_argument('-l', '--listen', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to listen on.')
parser.add_argument('-k', '--key', type=str, default='rt/turtle1/lidar',
                    help='The key expression to subscribe for LaserReadings.')
parser.add_argument('--intensity-treshold', type=float, default=250.0,
                    help='The intensity treshold.')
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

fig, ax = plt.subplots()
patch = ax.add_patch(Polygon([[0, 0]], color='lightgrey'))
line = ax.plot([], [], '.', color='black')[0]
center = ax.plot([0], [0], 'o', color='blue')[0]
ax.set_xlim(-4, 4)
ax.set_ylim(-4, 4)


def lidar_listener(sample):
    print('[DEBUG] Received frame: {}'.format(sample.key_expr))
    scan = LaserScan.deserialize(sample.payload)
    angles = list(map(lambda x: x*1j+cmath.pi/2j, np.arange(scan.angle_min, scan.angle_max, scan.angle_increment)))

    complexes = []
    for (angle, distance, intensity) in list(zip(angles, scan.ranges, scan.intensities)):
        complexes.append(distance * cmath.exp(angle) if intensity >= args.intensity_treshold else 1024 * cmath.exp(angle))
    X = [i.real for i in complexes]
    Y = [i.imag for i in complexes]
    XY = [[i.real, i.imag] for i in complexes]
    global line, patch
    patch.set_xy(XY)
    line.set_data(X, Y)

print("[INFO] Openning zenoh session...")
zenoh.init_logger()
z = zenoh.open(conf)

print("[INFO] Creating Subscriber on '{}'...".format(args.key))
sub = z.declare_subscriber(args.key, lidar_listener)

ani = FuncAnimation(fig, lambda _: None)
plt.show()

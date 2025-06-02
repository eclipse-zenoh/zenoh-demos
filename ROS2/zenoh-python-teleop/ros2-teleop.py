#
# Copyright (c) 2022 ZettaScale Technology Inc.
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
#

import argparse
import curses
import zenoh
import json
from dataclasses import dataclass
from pycdr2 import IdlStruct
from pycdr2.types import int8, int32, uint32, float64

@dataclass
class Vector3(IdlStruct, typename="Vector3"):
    x: float64
    y: float64
    z: float64


@dataclass
class Twist(IdlStruct, typename="Twist"):
    linear: Vector3
    angular: Vector3


@dataclass
class Time(IdlStruct, typename="Time"):
    sec: int32
    nanosec: uint32


@dataclass
class Log(IdlStruct, typename="Log"):
    stamp: Time
    level: int8
    name: str
    msg: str
    file: str
    function: str
    line: uint32


def main(stdscr):
    stdscr.refresh()

    # --- Command line argument parsing --- --- --- --- --- ---
    parser = argparse.ArgumentParser(
        prog='ros2-teleop',
        description='zenoh ros2 teleop example')
    parser.add_argument('--mode', '-m', dest='mode',
                        choices=['peer', 'client'],
                        type=str,
                        help='The zenoh session mode.')
    parser.add_argument('--connect', '-e', dest='connect',
                        metavar='ENDPOINT',
                        action='append',
                        type=str,
                        help='zenoh endpoints to connect to.')
    parser.add_argument('--listen', '-l', dest='listen',
                        metavar='ENDPOINT',
                        action='append',
                        type=str,
                        help='zenoh endpoints to listen on.')
    parser.add_argument('--config', '-c', dest='config',
                        metavar='FILE',
                        type=str,
                        help='A configuration file.')
    parser.add_argument('--cmd_vel', dest='cmd_vel',
                        default='rt/turtle1/cmd_vel',
                        type=str,
                        help='The "cmd_vel" ROS2 topic.')
    parser.add_argument('--rosout', dest='rosout',
                        default='rt/rosout',
                        type=str,
                        help='The "rosout" ROS2 topic.')
    parser.add_argument('--angular_scale', '-a', dest='angular_scale',
                        default='2.0',
                        type=float,
                        help='The angular scale.')
    parser.add_argument('--linear_scale', '-x', dest='linear_scale',
                        default='2.0',
                        type=float,
                        help='The linear scale.')

    args = parser.parse_args()
    conf = zenoh.Config.from_file(args.config) if args.config is not None else zenoh.Config()
    if args.mode is not None:
        conf.insert_json5('mode', json.dumps(args.mode))
    if args.connect is not None:
        conf.insert_json5('connect/endpoints', json.dumps(args.connect))
    if args.listen is not None:
        conf.insert_json5('listen/endpoints', json.dumps(args.listen))
    cmd_vel = args.cmd_vel
    rosout = args.rosout
    angular_scale = args.angular_scale
    linear_scale = args.linear_scale

    # zenoh-net code  --- --- --- --- --- --- --- --- --- --- ---

    # initiate logging
    zenoh.init_log_from_env_or("error")

    print("Openning session...")
    session = zenoh.open(conf)

    print("Subscriber on '{}'...".format(rosout))

    def rosout_callback(sample):
        log = Log.deserialize(sample.payload)
        print('[{}.{}] [{}]: {}'.format(log.stamp.sec,
                                        log.stamp.nanosec, log.name, log.msg))

    sub = session.declare_subscriber(rosout, rosout_callback)

    def pub_twist(linear, angular):
        print("Pub twist: {} - {}".format(linear, angular))
        t = Twist(linear=Vector3(x=float(linear), y=0.0, z=0.0),
                  angular=Vector3(x=0.0, y=0.0, z=float(angular)))
        session.put(cmd_vel, t.serialize())

    print("Waiting commands with arrow keys or space bar to stop. Press ESC or 'q' to quit.")
    while True:
        c = stdscr.getch()
        if c == curses.KEY_UP:
            pub_twist(1.0 * linear_scale, 0.0)
        elif c == curses.KEY_DOWN:
            pub_twist(-1.0 * linear_scale, 0.0)
        elif c == curses.KEY_LEFT:
            pub_twist(0.0, 1.0 * angular_scale)
        elif c == curses.KEY_RIGHT:
            pub_twist(0.0, -1.0 * angular_scale)
        elif c == 32:
            pub_twist(0.0, 0.0)
        elif c == 27 or c == ord('q'):
            break

    sub.undeclare()
    session.close()


curses.wrapper(main)

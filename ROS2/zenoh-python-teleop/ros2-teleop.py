#
# Copyright (c) 2021 ADLINK Technology Inc.
#
# This program and the accompanying materials are made available under the
# terms of the Eclipse Public License 2.0 which is available at
# http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
# which is available at https://www.apache.org/licenses/LICENSE-2.0.
#
# SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
#
# Contributors:
#   ADLINK zenoh team, <zenoh@adlink-labs.tech>
#

import sys
from datetime import datetime
import argparse
import curses
import zenoh
from zenoh.net import config, SubInfo, Reliability, SubMode
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


@cdr
class Time:
    sec: int32
    nanosec: uint32


@cdr
class Log:
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
        prog='zn_sub',
        description='zenoh-net sub example')
    parser.add_argument('--mode', '-m', dest='mode',
                        choices=['peer', 'client'],
                        type=str,
                        help='The zenoh session mode.')
    parser.add_argument('--peer', '-e', dest='peer',
                        metavar='LOCATOR',
                        action='append',
                        type=str,
                        help='Peer locators used to initiate the zenoh session.')
    parser.add_argument('--listener', '-l', dest='listener',
                        metavar='LOCATOR',
                        action='append',
                        type=str,
                        help='Locators to listen on.')
    parser.add_argument('--config', '-c', dest='config',
                        metavar='FILE',
                        type=str,
                        help='A configuration file.')
    parser.add_argument('--cmd_vel', dest='cmd_vel',
                        default='/rt/turtle1/cmd_vel',
                        type=str,
                        help='The "cmd_vel" ROS2 topic.')
    parser.add_argument('--rosout', dest='rosout',
                        default='/rt/rosout',
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
    conf = zenoh.config_from_file(
        args.config) if args.config is not None else {}
    if args.mode is not None:
        conf["mode"] = args.mode
    if args.peer is not None:
        conf["peer"] = ",".join(args.peer)
    if args.listener is not None:
        conf["listener"] = ",".join(args.listener)
    cmd_vel = args.cmd_vel
    rosout = args.rosout
    angular_scale = args.angular_scale
    linear_scale = args.linear_scale

    # zenoh-net code  --- --- --- --- --- --- --- --- --- --- ---

    # initiate logging
    zenoh.init_logger()

    print("Openning session...")
    session = zenoh.net.open(conf)

    print("Subscriber on '{}'...".format(rosout))
    sub_info = SubInfo(Reliability.Reliable, SubMode.Push)

    def rosout_callback(sample):
        log = Log.deserialize(sample.payload)
        print('[{}.{}] [{}]: {}'.format(log.stamp.sec,
                                        log.stamp.nanosec, log.name, log.msg))

    sub = session.declare_subscriber(rosout, sub_info, rosout_callback)

    def pub_twist(linear, angular):
        print("Pub twist: {} - {}".format(linear, angular))
        t = Twist(linear=Vector3(x=linear, y=0.0, z=0.0),
                  angular=Vector3(x=0.0, y=0.0, z=angular))
        session.write(cmd_vel, t.serialize())

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
        elif c == 27 or c == ord('q'):
            break

    sub.undeclare()
    session.close()


curses.wrapper(main)

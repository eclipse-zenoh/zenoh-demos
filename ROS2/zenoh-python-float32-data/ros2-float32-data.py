#
# Copyright (c) 2024 ZettaScale Technology Inc.
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
import zenoh
import random
import time
import json

from dataclasses import dataclass
from pycdr2 import IdlStruct
from pycdr2.types import int32, uint32, float32


@dataclass
class Time(IdlStruct, typename="Time"):
    sec: int32
    nanosec: uint32

@dataclass
class Float32(IdlStruct, typename="std_msgs/msg/Float32"):
    data: float32


def main():
    # --- Command line argument parsing ---
    parser = argparse.ArgumentParser(
        prog='random_float32_publisher',
        description='zenoh random Float32 publisher example')
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
    parser.add_argument('--topic', '-t', dest='topic',
                        default='random_float32',
                        type=str,
                        help='The topic to publish Float32 data.')
    parser.add_argument('--float32_data', '-f', dest='float32_data',
                        default='10.0',
                        type=float,
                        help='The float32_data of publishing.')

    args = parser.parse_args()
    conf = zenoh.Config.from_file(args.config) if args.config is not None else zenoh.Config()
    if args.mode is not None:
        conf.insert_json5(zenoh.config.MODE_KEY, json.dumps(args.mode))
    if args.connect is not None:
        conf.insert_json5(zenoh.config.CONNECT_KEY, json.dumps(args.connect))
    if args.listen is not None:
        conf.insert_json5(zenoh.config.LISTEN_KEY, json.dumps(args.listen))
    topic = args.topic
    float32_data = args.float32_data

    # zenoh-net session initialization
    print("Opening session...")
    session = zenoh.open(conf)

    print(f"Publishing random Float32 data on topic '{topic}' at {float32_data}.")

    def pub_float32(data):
        print("Pub float32: {}".format(data))
        t = Float32(data=float(data))
        session.put(topic, t.serialize())

    try:
        interval = 1.0 / float32_data
        while True:
            random_value = random.uniform(-10, 10.0)  # Generate random float between -100.0 and 100.0
            pub_float32(random_value)
            print(f"Published: {random_value} to topic '{topic}'")
            time.sleep(interval)
    except KeyboardInterrupt:
        print("Shutting down...")
    finally:
        session.close()

if __name__ == "__main__":
    main()

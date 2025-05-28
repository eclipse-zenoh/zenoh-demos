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
#   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
#

import sys
import time
import argparse
import itertools
import json
import zenoh
from zenoh import config, Encoding

# --- Command line argument parsing --- --- --- --- --- ---
parser = argparse.ArgumentParser(
    prog='z_pub',
    description='zenoh pub example')
parser.add_argument('--v', '-v', dest='vehicle',
                    choices=[1,2,3,4],
                    type=int,
                    help='The vehicle to send')

args = parser.parse_args()
conf = zenoh.Config()
conf.insert_json5(zenoh.config.MODE_KEY, json.dumps("client"))
conf.insert_json5(zenoh.config.CONNECT_KEY, json.dumps(["tcp/3.71.106.121:7447"]))

vehicle = args.vehicle - 1

cars = [
  {
    'position': { 'lat': 48.864950, 'lng': 2.349000 },
    'speed': 10,
    'color': '#ff0000',
    'id': 'luca',
    'kind':'car'
  },
  {
    'position': { 'lat': 48.865250, 'lng': 2.347200 },
    'speed': 10,
    'color': '#000000',
    'id': 'test',
    'kind':'whoknows'
  },
  {
    'position': { 'lat': 48.864750, 'lng': 2.349200 },
    'speed': 20,
    'color': '#ffffff',
    'id': 'angelo',
    'kind':"motorbike"
  },
  {
    'position': { 'lat': 48.864550, 'lng': 2.349550 },
    'speed': 30,
    'color': '#0000ff',
    'id': 'gabriele',
    "kind":"motorbike",
  },
    {
    'position': { 'lat': 48.865500, 'lng': 2.349550 },
    'speed': 40,
    'color': '#ff00ff',
    'id': 'steven',
    "kind":"car",
  }
]

key = f'demo/vehicles/{cars[vehicle]['id']}'


def main():
    # initiate logging
    zenoh.init_log_from_env_or("error")

    print("Opening session...")
    session = zenoh.open(conf)

    print(f"Declaring Publisher on '{key}'...")
    pub = session.declare_publisher(key)

    print("Press CTRL-C to quit...")
    while True:
        time.sleep(0.5)
        buf = json.dumps(cars[vehicle]).encode("utf-8")
        print(f"Putting Data ('{key}': '{buf}')...")
        pub.put(value=buf, encoding=Encoding.APP_JSON())

        cars[vehicle]['position']['lat'] += 0.00001
        cars[vehicle]['position']['lng'] += 0.00001
        # cars[vehicle]['position']['timestamp'] = time.time()

    pub.undeclare()
    session.close()

main()


# run as: python3 ~/Workspace/zettascale/zenoh/zenoh-demos/zenoh-map-dashboard/pub_positions.py -v 0

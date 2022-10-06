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
import argparse
import json
import random
import time
import zenoh 

parser = argparse.ArgumentParser(
    prog='z_pub_rand',
    description='zenoh random integer publication example')
parser.add_argument('--mode', '-m', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('--connect', '-e', type=str, metavar='ENDPOINT', action='append',   
                    help='Endpoints to connect to.')
parser.add_argument('--listen', '-l', type=str, metavar='ENDPOINT', action='append',
                    help='Endpoints to listen on.')
parser.add_argument('-k', '--key', type=str, default='demo/random',
                    help='The key expression to publish onto.')
parser.add_argument('-a', '--max', type=int, default=100,
                    help='The maximum generated value.')
parser.add_argument('-d', '--delay', type=float, default=1.0,
                    help='The delay between each publication in seconds.')
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

z = zenoh.open(conf)
while True:
    x = random.randint(0, args.max)
    print("Putting Data ('{}': {})...".format(args.key, x))
    z.put(args.key, x)

    time.sleep(args.delay)
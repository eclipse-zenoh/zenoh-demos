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

# --- Command line argument parsing --- --- --- --- --- ---
parser = argparse.ArgumentParser(
    prog='z_pub',
    description='zenoh pub example')

args = parser.parse_args()
conf = zenoh.Config()
conf.insert_json5("mode", json.dumps("client"))
conf.insert_json5("connect/endpoints", json.dumps(["tcp/127.0.0.1:7447"]))



key = 'demo/tracker/alert/distance'


def main():
    # initiate logging
    zenoh.try_init_log_from_env()

    print("Opening session...")
    session = zenoh.open(conf)

    print(f"Declaring Publisher on '{key}'...")
    pub = session.declare_publisher(key, encoding=zenoh.Encoding.APPLICATION_JSON)

    print("Press CTRL-C to quit...")
    while True:
        buf = json.dumps({'ida':'test', 'idb':'test',"distance":"12", 'kind':'DANGER'}).encode("utf-8")
        print(f"Putting Data ('{key}': '{buf}')...")
        pub.put(buf)
        time.sleep(5)

        

    pub.undeclare()
    session.close()

main()


# run as: python3 ~/Workspace/zettascale/zenoh/zenoh-demos/zenoh-map-dashboard/pub_positions.py -v 0

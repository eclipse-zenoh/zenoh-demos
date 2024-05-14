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
from datetime import datetime
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
import numpy
import time
import json
import zenoh

# --- Command line argument parsing --- --- --- --- --- ---
parser = argparse.ArgumentParser(
    prog='z_plot',
    description='zenoh plotting example')
parser.add_argument('--mode', '-m', type=str, choices=['peer', 'client'], 
                    help='The zenoh session mode.')
parser.add_argument('--connect', '-e', type=str, metavar='ENDPOINT', action='append',   
                    help='Endpoints to connect to.')
parser.add_argument('--listen', '-l', type=str, metavar='ENDPOINT', action='append',
                    help='Endpoints to listen on.')
parser.add_argument('-k', '--key', type=str, default='demo/random',
                    help='The key expression to subscribe to.')
parser.add_argument('-i', '--history', type=float, default=10.0,
                    help='The history depth in seconds.')
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

lines = {}
fig, ax = plt.subplots()
ax.xaxis.axis_date()

def listener(sample):
    if not str(sample.key_expr) in lines:
        lines[str(sample.key_expr)] = ax.plot([], [], '-o', label=str(sample.key_expr))[0]
    now = time.time()
    xdata, ydata = lines[str(sample.key_expr)].get_data()
    xdata = numpy.append(xdata, datetime.fromtimestamp(now if sample.timestamp is None else sample.timestamp.time/4294967295))
    ydata = numpy.append(ydata, float(sample.payload.decode("utf-8")))
    lines[str(sample.key_expr)].set_data(zip(*filter(lambda t: t[0].timestamp() > now - args.history, zip(xdata, ydata))))

def update(_):
    if len(lines):
        ax.axes.relim()
        ax.axes.autoscale_view(True,True,True)
        ax.legend(loc=2)

zenoh.init_logger()

print("Openning session...")
z = zenoh.open(conf)

print("Declaring Subscriber on '{}'...".format(args.key))
sub = z.declare_subscriber(args.key, listener)

ani = FuncAnimation(fig, update)
plt.show()

import zenoh
import sys
import json
import argparse
import time

parser = argparse.ArgumentParser(
    prog='detect_faces',
    description='zenoh face recognition example face detector')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--peer', type=str, metavar='LOCATOR', action='append',
                    help='Peer locators used to initiate the zenoh session.')
parser.add_argument('-l', '--listener', type=str, metavar='LOCATOR', action='append',
                    help='Locators to listen on.')
parser.add_argument('-d', '--dataset', required=True,
                    help='vectors dataset location')
parser.add_argument('-p', '--prefix', type=str, default='/demo/facerecog',
                    help='resources prefix')
parser.add_argument('-c', '--config', type=str, metavar='FILE',
                    help='A zenoh configuration file.')

args = parser.parse_args()
conf = zenoh.config_from_file(args.config) if args.config is not None else zenoh.Config()
if args.mode is not None:
    conf.insert_json5("mode", json.dumps(args.mode))
if args.peer is not None:
    conf.insert_json5("peers", json.dumps(args.peer))
if args.listener is not None:
    conf.insert_json5("listeners", json.dumps(args.listener))

f = open(args.dataset)
faces = json.load(f)

print('[INFO] Open zenoh session...')
zenoh.init_logger()
z = zenoh.open(conf)
time.sleep(0.5)

# If not yet existing, add a memory storage that will store the dataset
try:
    print('{}'.format(z.info()['info_router_pid'].split(',')[0]))
    storage_admin_path = '/@/router/{}/config/plugins/storages/backends/memory/storages/facerecog-store'.format(
        z.info()['info_router_pid'].split(',')[0])
    if not z.get(storage_admin_path):
        key_expr = '{}/vectors/**'.format(args.prefix)
        print('Add storage: on {}'.format(key_expr))
        z.put(storage_admin_path, zenoh.Value.Json(json.dumps({'key_expr': key_expr})))
        time.sleep(1)
except:
    e = sys.exc_info()[0]
    print('Error creating storage: {}'.format(e))


for k, vs in faces.items():
    for j, v in enumerate(vs):
        uri = '{}/vectors/{}/{}'.format(args.prefix, k, j)
        print('> Inserting face {}'.format(uri))
        z.put(uri, zenoh.Value.Json(json.dumps(v)))

z.close()

print('[INFO] Done.')

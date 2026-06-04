import argparse
import time
import cv2
import zenoh
import numpy as np
import json

# Argument parser
parser = argparse.ArgumentParser(
    prog='zdisplay',
    description='zenoh video display example')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--connect', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to connect to.')
parser.add_argument('-l', '--listen', type=str, metavar='ENDPOINT', action='append',
                    help='zenoh endpoints to listen on.')
parser.add_argument('-d', '--delay', type=float, default=0.05,
                    help='delay between each frame in seconds')
parser.add_argument('-k', '--key', type=str, default='demo/zcam',
                    help='key expression')
parser.add_argument('-c', '--config', type=str, metavar='FILE',
                    help='A zenoh configuration file.')

args = parser.parse_args()

# Zenoh session setup
conf = zenoh.Config.from_file(args.config) if args.config is not None else zenoh.Config()

# Setting Zenoh mode (peer or client)
if args.mode is not None:
    conf.insert_json5("mode", json.dumps(args.mode))

# Connect to the provided endpoints, or use a default if not provided
if args.connect is not None:
    conf.insert_json5("connect", json.dumps(args.connect))
else:
    # Default to localhost if no connect endpoint is specified
    print("[INFO] No connect endpoint specified, using default endpoint (tcp://127.0.0.1:7447).")

# Listen to the provided endpoints if any
if args.listen is not None:
    conf.insert_json5("listen", json.dumps(args.listen))

# Dictionary to store camera images
cams = {}

def frames_listener(sample):
    """Callback function for Zenoh subscriber"""
    # Convert received image data (ZBytes type) to numpy array
    npImage = np.frombuffer(bytes(sample.payload), dtype=np.uint8)
    matImage = cv2.imdecode(npImage, 1)

    # Update the latest camera image (separate management for RGB and Depth)
    cams[sample.key_expr] = matImage

# Open Zenoh session
print('[INFO] Open zenoh session...')
z = zenoh.open(conf)

# Declare subscribers for RGB and Depth
sub_rgb = z.declare_subscriber(args.key + "/rgb", frames_listener)
sub_depth = z.declare_subscriber(args.key + "/depth", frames_listener)

while True:
    # Display all camera images in separate windows
    for cam_key, cam_img in cams.items():
        # Convert cam_key to string
        cam_key_str = str(cam_key)

        if 'rgb' in cam_key_str:
            cv2.imshow(f"RGB Camera: {cam_key_str}", cam_img)  # Display RGB camera
        elif 'depth' in cam_key_str:
            cv2.imshow(f"Depth Camera: {cam_key_str}", cam_img)  # Display Depth camera

    # Exit on ESC key
    key = cv2.waitKey(1) & 0xFF
    if key == 27:  # Exit on ESC key
        break

    # Add delay before receiving the next frame
    time.sleep(args.delay)

# Close the windows
cv2.destroyAllWindows()

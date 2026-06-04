import zenoh
import cv2
import numpy as np
import pyrealsense2 as rs
import argparse
import json
import time

# Argument settings
parser = argparse.ArgumentParser(
    prog='zrs_capture',
    description='Zenoh RealSense RGB and Depth capture example')
parser.add_argument('-m', '--mode', type=str, choices=['peer', 'client'],
                    help='The zenoh session mode.')
parser.add_argument('-e', '--connect', type=str, metavar='ENDPOINT', action='append',
                    help='Zenoh endpoints to connect to.')
parser.add_argument('-l', '--listen', type=str, metavar='ENDPOINT', action='append',
                    help='Zenoh endpoints to listen on.')
parser.add_argument('-k', '--key', type=str, default='demo/zcam',
                    help='Key expression')
parser.add_argument('-c', '--config', type=str, metavar='FILE',
                    help='A Zenoh configuration file.')
args = parser.parse_args()

# Zenoh configuration
conf = zenoh.Config.from_file(args.config) if args.config is not None else zenoh.Config()
z = zenoh.open(conf)

# RealSense pipeline configuration
pipeline = rs.pipeline()
config = rs.config()
config.enable_stream(rs.stream.color, 640, 480, rs.format.bgr8, 30)  # RGB
config.enable_stream(rs.stream.depth, 640, 480, rs.format.z16, 30)   # Depth

pipeline.start(config)

print("[INFO] Open RealSense camera...")

# Publish frames to Zenoh
def publish_frames():
    while True:
        # Wait for frames
        frames = pipeline.wait_for_frames()
        color_frame = frames.get_color_frame()
        depth_frame = frames.get_depth_frame()

        if not color_frame or not depth_frame:
            continue

        # Get RGB image
        rgb_image = np.asanyarray(color_frame.get_data())
        _, jpeg_rgb = cv2.imencode('.jpg', rgb_image)
        z.put(args.key + "/rgb", jpeg_rgb.tobytes())

        # Get Depth image
        depth_image = np.asanyarray(depth_frame.get_data())

        # Convert Depth data to color image (apply color map)
        depth_colormap = cv2.applyColorMap(cv2.convertScaleAbs(depth_image, alpha=0.03), cv2.COLORMAP_JET)

        # Encode Depth image in JPEG format
        _, jpeg_depth = cv2.imencode('.jpg', depth_colormap)
        z.put(args.key + "/depth", jpeg_depth.tobytes())

        time.sleep(0.05)

# Send frames
publish_frames()

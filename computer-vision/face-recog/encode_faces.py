# USAGE
# When encoding on laptop, desktop, or GPU (slower, more accurate):
# python3 encode_faces.py --dataset <dataset-path> --detection-method cnn
# When encoding on Raspberry Pi (faster, less accurate):
# python3 encode_faces.py --dataset <dataset-path> -detection-method hog

# import the necessary packages
from imutils import paths
import face_recognition
import argparse
import pickle
import cv2
import os
import json

# construct the argument parser and parse the arguments
parser = argparse.ArgumentParser(
    prog='encode_faces',
    description='zenoh face recognition example face encoder')
parser.add_argument('-i', '--dataset', required=True,
                    help='path to input directory of faces + images')
parser.add_argument('-o', '--output', required=True,
                    help='path to output file')
parser.add_argument('-d', '--detection-method', type=str, default='cnn',
                    help='face detection model to use: either `hog` or `cnn`')
args = vars(parser.parse_args())

# grab the paths to the input images in our dataset
print('[INFO] Quantify faces...')
imagePaths = list(paths.list_images(args['dataset']))

# initialize the list of known encodings and known names
face_db = {}

# loop over the image paths
for (i, imagePath) in enumerate(imagePaths):
    # extract the person name from the image path
    print('[INFO] process image {}/{} ({})'.format(
        i + 1, len(imagePaths), imagePath))
    name = imagePath.split(os.path.sep)[-2]

    # load the input image and convert it from RGB (OpenCV ordering)
    # to dlib ordering (RGB)
    image = cv2.imread(imagePath)
    rgb = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)

    # detect the (x, y)-coordinates of the bounding boxes
    # corresponding to each face in the input image
    boxes = face_recognition.face_locations(
        rgb, model=args['detection_method'])

    # compute the facial embedding for the face
    encodings = face_recognition.face_encodings(rgb, boxes)
    # loop over the encodings
    for encoding in encodings:

        # add each encoding + name to our set of known names and
        # encodings
        elist = encoding.tolist()
        if name in face_db.keys():
            face_db[name].append(elist)
        else:
            face_db[name] = [elist]


# dump the facial encodings + names to disk
print('[INFO] Serialize encodings...')

f = open(args['output'], 'w')
json.dump(face_db, f)
f.close()

print('[INFO] Done.')

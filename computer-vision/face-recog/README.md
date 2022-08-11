# Eclipse zenoh Face Recognition Demo
This is a relatively simple demo that shows how zenoh can be used to do face recognition as well as notification of recognised faces.

## Pre-requisite
Python 3, pip3 and the zenoh-python api.
Install the required python modules:

        $ pip3 install jsonschema jsonpickle argcomplete imutils opencv-python opencv-contrib-python face_recognition eclipse-zenoh


## Step I -- Start a zenoh instance
The simplest way to run the example is to use our online instance of zenoh available at **demo.zenoh.io**.
For this, you just need to use the option `-e tcp/demo.zenoh.io:7447` with the python scripts,
as described in the nex steps.

Otherwise you can run a local zenoh instance. See the instructions to run one within a Docker container here:
https://github.com/eclipse-zenoh/zenoh/#how-to-test-it.  
In this case, you don't need to use the `-e` option with the python scripts.
The zenoh-python library will automatically discover the zenoh instance via UDP multicast.

## Step II -- Prepare Your Data Set (optional)
The directory **dataset** contains some existing data-set for some of the members of the
Advanced Technology Office in ADLINK, some famous Tennis Player, Soccer Player and Hollywood starts.
The data-set are actually collections of faces pictures organized in subdirectories:
**dataset/*category*/*name*/\*.jpg**  
These data-set have already been transformed into face-signature databases (JSON format) in face-sig-db. 

If you want to generate those databases again (for instance because you added more pictures or a new data-set of pictures), it can be done as follows:

        $ python3 encode_faces.py --dataset dataset/tennis --detection-method cnn -o face-sig-db/tennis-db.json

## Step III -- Load the Data Set on zenoh
Now you should load each database (JSON file) on zenoh, to do so, execute the following commands for each database:

        $ python3 load_face_db.py -d face-sig-db/tennis-db.json 

For using our demo.zenoh.io do:

        $ python3 load_face_db.py -d face-sig-db/tennis-db.json -e tcp/demo.zenoh.io:7447

## Step IV -- Run the video capture component
This component reads frames from the camera and publishes them to zenoh.

        $ python3 capture_video.py

For using our demo.zenoh.io do:

        $ python3 capture_video.py -e tcp/demo.zenoh.io:7447

## Step V -- Run the face detection component
This component subscribes to video frames from the video capture component, detects faces and publishes the faces images to zenoh.

        $ python3 detect_faces.py

For using our demo.zenoh.io do:

        $ python3 detect_faces.py -e tcp/demo.zenoh.io:7447

## Step VI -- Run the face recognition component
This component subscribes to faces images from the detecton component, and to face signatures from the dataset on zenoh, identifies received faces and publishes identifications to zenoh.

        $ python3 recognize_faces.py

For using our demo.zenoh.io do:

        $ python3 recognize_faces.py -e tcp/demo.zenoh.io:7447

## Step VII -- Run the display component
This component subscribes to faces images from the detecton component, and to identifications from the face recognition component on zenoh and displays them.

        $ python3 display_faces.py

For using our demo.zenoh.io do:

        $ python3 display_faces.py -e tcp/demo.zenoh.io:7447

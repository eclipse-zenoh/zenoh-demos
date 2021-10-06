#!/bin/bash
move() # <app> <port> [backup_ports]
{
  local app=$1
  shift
  ports=[\"tcp/127.0.0.1:$(echo $* | sed "s% %\",\"tcp/127.0.0.1:%g")\"]
  echo "curl -X PUT -d $ports http://localhost:8001/demo/zcam/$app/conf/peers"
  curl -X PUT -d $ports http://localhost:8001/demo/zcam/$app/conf/peers
}
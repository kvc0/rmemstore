#!/bin/bash

sudo apt update
sudo apt install python3-venv python3-pip

source bin/activate

pip install protobuf

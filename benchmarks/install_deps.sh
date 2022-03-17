#!/bin/sh

DISTRO=$(cat /etc/*-release | awk -F= '/^NAME="/{gsub(/"/, "", $2);print $2}')

if [ "$DISTRO" = "Fedora Linux" ]; then
    sudo dnf install python3-numpy
elif [ "$DISTRO" = "Ubuntu" ]; then
    sudo apt-get install python3-numpy
else
    echo "Unsupported distribution, numpy will be install from pip."
    python -m pip install --user Cython
fi

snap install mosquitto

python -m pip install --user wheel
python3 setup.py install --user

git clone https://github.com/eclipse/paho.mqtt-sn.embedded-c
cd paho.mqtt-sn.embedded-c/MQTTSNGateway 
./build.sh udp

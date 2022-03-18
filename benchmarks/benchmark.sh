#!/bin/sh

# Absolute path to this script, e.g. /home/user/bin/foo.sh
SCRIPT=$(readlink -f "$0")
# Absolute path this script is in, thus /home/user/bin
SCRIPT_PATH=$(dirname "$SCRIPT")

# If the 
if [ ! -d "$SCRIPT_PATH/paho.mqtt-sn.embedded-c" ]; then
    echo "Installing dependencies"

    DISTRO=$(cat /etc/*-release | awk -F= '/^NAME="/{gsub(/"/, "", $2);print $2}')

    if [ "$DISTRO" = "Fedora Linux" ]; then
        sudo dnf install python3-numpy
    elif [ "$DISTRO" = "Ubuntu" ]; then
        sudo apt-get install python3-numpy
    else
        echo "Unsupported distribution, numpy will be installed from pip."
        python3 -m pip install --user Cython
    fi

    python3 -m pip install --user wheel
    python3 -m pip install $SCRIPT_PATH

    sudo snap install mosquitto

    pushd "$SCRIPT_PATH"
    git clone https://github.com/eclipse/paho.mqtt-sn.embedded-c
    cd "$SCRIPT_PATH/paho.mqtt-sn.embedded-c/MQTTSNGateway"
    ./build.sh udp
    popd

fi

# Note: Can pass the interface to use as the first argument
INTERFACE=${1:-"wlp5s0"}

# Clear interface beforehand
sudo tc qdisc del dev $INTERFACE root netem 2>/dev/null || true

# Have a delay of 100ms with 10ms of variance with the delay distributed in a
# normal distribution.
# Also add a 0.3% of packet loss with a 25% of correlation (a random element
# depends 25% on the last one).
# A 1% of packets will be duplicated.
# A 0.1% of packets will be corrupted.
# 25% of packets will be sent immediately, others will have the normal delay
sudo tc qdisc add dev $INTERFACE root netem \
delay 100ms 10ms \
loss 0.3% 25% \
duplicate 1% \
corrupt 0.1% \
reorder 25% 50%

# We'll consider that mosquitto MQTT is up and running
python3 "$SCRIPT_PATH/dummy_server.py" & \
"$SCRIPT_PATH/paho.mqtt-sn.embedded-c/MQTTSNGateway/bin/MQTT-SNGateway" -f "$SCRIPT_PATH/MQTTSN.conf" & \
python3 "$SCRIPT_PATH/main.py"

sudo tc qdisc del dev $INTERFACE root

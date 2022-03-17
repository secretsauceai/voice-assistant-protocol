#!/bin/sh

INTERFACE = "wlp5s0"

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
delay 100ms 10ms distribution normal \
loss 0.3% 25% \
duplicate 1% \
corrupt 0.1% \
reorder 25% 50%

# We'll consider that mosquitto MQTT is up and running
python3 dummy_server.py & \
paho.mqtt-sn.embedded-c/bin/MQTT-SNGateway & \
python3 main.py

sudo tc qdisc del dev $INTERFACE root

#!/usr/bin/env python3

import benchwork
import protos
from os import urandom

SAMPLE_RATE = 48000
SAMPLES_PER_MS = SAMPLE_RATE / 1000

def make_sound(ms: int = 0) -> bytes:
    return bytearray(urandom(ms * SAMPLES_PER_MS))

@benchwork.benchmark(with_classes=protos.all_protos)
def voice_send(proto: protos.VaProto):
    proto.send_message("/asis_api/voice", {"voice": make_sound(ms=200)})

@benchwork.benchmark(with_classes=protos.all_protos)
def sound_send(proto: protos.VaProto):
    proto.send_message("/asis_api/sound", {"sound": make_sound(ms=2_000)})

@benchwork.benchmark(with_classes=protos.all_protos)
def large_sound(proto: protos.VaProto):
    proto.recv_message("/asis_api/sound", {"sound": make_sound(ms=50_000)})

@benchwork.benchmark(with_classes=protos.all_protos)
def skill_answer(proto: protos.VaProto):
    proto.send_message("/asis_api/data", {"TODO": "DATA"})

benchwork.run()
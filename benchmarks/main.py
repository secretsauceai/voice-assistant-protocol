#!/usr/bin/env python3

import benchwork
import protos
from os import urandom

SAMPLE_RATE = 48000
SAMPLES_PER_MS = int(SAMPLE_RATE / 1000)

client = "org.mycompany.myclient.134567"

def make_sound(ms: int = 0) -> bytes:
    return bytearray(urandom(ms * SAMPLES_PER_MS))

@benchwork.benchmark(with_classes=protos.all_protos)
def voice_send(proto: protos.VaProto):
    proto.send_message("/asis_api/voice/", client, make_sound(ms=200))

@benchwork.benchmark(with_classes=protos.all_protos)
def sound_send(proto: protos.VaProto):
    proto.send_message("/asis_api/sound/", client, make_sound(ms=2_000))

@benchwork.benchmark(with_classes=protos.all_protos)
def large_sound(proto: protos.VaProto):
    proto.send_message("/asis_api/sound/", client, make_sound(ms=50_000))

@benchwork.benchmark(with_classes=protos.all_protos)
def skill_answer(proto: protos.VaProto):
    proto.send_message("/asis_api/intent/request_weather", client, {
        "input": "what is the weather like in london, england tomorrow?",
        "intent": {
            "intentName": "request_weather",
            "confidenceScore": 0.8564,
        },
        "slots": [
            {
                "entity": "location",
                "slot": "place",
                "confidence": 0.687,
                "rawValue": "london, england",
                "value": {
                    "latitude": 51.5073,
                    "longitude": -0.1277,
                },
                "range": {
                    "start": 28,
                    "end": 43
                }
            }
        ],
        "id": "13546789",
        "siteId": client,
        "sessionId": "1234a56d3c679e",
        "asrConfidence": 0.678,

    })

benchwork.run()
from typing import Any, Dict, Protocol
import asyncio

import paho.mqtt.client as mqtt
import mqttsn.client
import aiocoap
import msgpack
from flatbuffers import flexbuffers
from mapbuffer import MapBuffer
import mycroft_bus_client as mycroft
from hivemind_bus_client import HiveMessageBusClient

MQTT_SERVER_IP = ""
MQTT_SEVER_PORT = 1883

MQTTSN_SERVER_IP = ""
MQTTSN_SERVER_PORT = 1883

COAP_SERVER_IP = ""
COAP_SERVER_PORT = 5683

class VaProto(Protocol):
    def send_message(self, dest: str, data: Dict[str, Any]):
        # ...
        pass

class MqttBase(VaProto):
    def __init__(self):
        self.client = mqtt.Client()

        self.client.connect(MQTT_SERVER_IP, MQTT_SEVER_PORT)
    
    def send_message(self, dest: str, data: Dict[str, Any]):
        msg = self.client.publish(dest, paylooad=self.transform_payload(data))
        msg.wait_for_publish()

    def transform_payload(self, data: Dict[str, Any]) -> str:
        raise NotImplementedError()

class MqttSnBase(VaProto):
    def __init__(self):
        self.client = mqttsn.client.Client(MQTTSN_SERVER_IP, MQTTSN_SERVER_PORT)
        self.client.connect()

    def send_message(self, dest: str, data: Dict[str, Any]):
        self.client.publish(dest, self.transform_payload(data))
    
    def transform_payload(self, data: Dict[str, Any]) -> str:
        raise NotImplementedError()

class CoApBase(VaProto):
    def __init__(self):
        self.client = asyncio.get_event_loop().run_until_complete(aiocoap.Context.create_client_context())
    
    def send_message(self, dest: str, data: Dict[str, Any]):
        request = aiocoap.Message(code=aiocoap.POST, uri=COAP_SERVER_IP + '/' + dest, payload=self.transform_payload(data))
        asyncio.get_event_loop().run_until_complete(self.client.request(request))    
    
    def transform_payload(self, data: Dict[str, Any]) -> str:
        raise NotImplementedError()

# Protocols themselves

## MQTT
class MqttFlexbuf(MqttBase):
    def transform_payload(self, data: Dict[str, Any]) -> str:
        return flexbuffers.Dumps(data)

class MqttMapBuf(MqttBase):
    def transform_payload(self, data: Dict[str, Any]) -> str:
        return MapBuffer(data).tobytes()

class MqttMessagePack(MqttBase):
    def transform_payload(self, data: Dict[str, Any]) -> str:
        return msgpack.packb(data)

## MQTT-SN
class MqttSnFlexbuf(MqttSnBase):
    def transform_payload(self, data: Dict[str, Any]) -> str: 
        return flexbuffers.Dumps(data)

class MqttSnMapBuf(MqttSnBase):
    def transform_payload(self, data: Dict[str, Any]) -> str:
        return MapBuffer(data).tobytes()

class MqttSnMessagePack(MqttSnBase):
    def transform_payload(self, data: Dict[str, Any]) -> str:
        return msgpack.packb(data)

## CoAP
class CoApFlexbuf(VaProto):
    def transform_payload(self, data: Dict[str, Any]) -> str:
        return flexbuffers.Dumps(data)

class CoApMapBuf(VaProto):
    def transform_payload(self, data: Dict[str, Any]) -> str:
        return MapBuffer(data).tobytes()

class CoApMessagePack(VaProto):
    def transform_payload(self, data: Dict[str, Any]) -> str:
        return msgpack.packb(data)

## Others
class Hermes(VaProto):
    """ Implementation of Hermes, which is Mqtt + JSON. They sometimes just send
        a binary payload because JSON is not compatible with binary."""
    def __init__(self):
        self.client = mqtt.Client()

        self.client.connect(MQTT_SERVER_IP, MQTT_SEVER_PORT)

    def send_message(self, dest: str, data: Dict[str, Any]):
        msg = self.client.publish(self.transform_dest(dest), paylooad=self.transform_payload(data))
        msg.wait_for_publish()

    def transform_payload(self, data: Dict[str, Any]) -> str:
        raise NotImplementedError()
    
    @staticmethod
    def transform_dest(dest: str) -> str:
        if dest == "/asis_api/voice":
            return "/hermes/..."
        elif dest == "/asis_api/sound":
            return "/hermes/..."
        elif dest == "/asis_api/data":
            return "/hermes/..."

        return "UNKNOWN!!"

        

class HiveMind(VaProto):
    def __init__(self):
        key = "super_secret_access_key"
        crypto_key = "ivf1NQSkQNogWYyr"

        self.client = HiveMessageBusClient(key, crypto_key=crypto_key, ssl=False)

    def send_message(self, dest: str, data: Dict[str, Any]):
        mycroft_msg = mycroft.Message(dest, data)
        self.client.emit_mycroft(mycroft_msg)


all_protos = [
    MqttFlexbuf, MqttMapBuf, MqttMessagePack,
    MqttSnFlexbuf, MqttSnMapBuf, MqttSnMessagePack, 
    CoApFlexbuf, CoApMapBuf, CoApMessagePack,
    Hermes, HiveMind
]

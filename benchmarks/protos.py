from ast import Call
from typing import Any, Callable, Dict, Protocol, Union
import asyncio

# Transport protocols
import aiocoap
import mqttsn.client
import paho.mqtt.client as mqtt

# Message formats
import msgpack
from flatbuffers import flexbuffers
from hivemind_bus_client import HiveMessageBusClient
import json
from mapbuffer import MapBuffer
import mycroft_bus_client as mycroft


MQTT_SERVER_IP = ""
MQTT_SEVER_PORT = 1883

MQTTSN_SERVER_IP = ""
MQTTSN_SERVER_PORT = 1883

COAP_SERVER_IP = ""
COAP_SERVER_PORT = 5683

SendData = Union[Dict[str, Any],  bytes]

def encapsulate_bin(data: SendData, f: Callable[[Dict[str, Any]], bytes]) -> bytes:
    if isinstance(data, dict):
        return f(data)
    return f({"voice": data})

class VaProto(Protocol):
    """Interface for all protocol implementations. Not to be used directly"""

    def send_message(self, path: str, dest: str, data: SendData):
        raise NotImplementedError()

class MqttBase(VaProto):
    """Base class for MQTT-based protocols. Not to be used directly"""

    def __init__(self):
        self.client = mqtt.Client()

        self.client.connect(MQTT_SERVER_IP, MQTT_SEVER_PORT)
    
    def send_message(self, path: str, dest: str, data: SendData):
        msg = self.client.publish(path + "/" + dest, paylooad=self.transform_payload(data))
        msg.wait_for_publish()

    def transform_payload(self, data: SendData) -> bytes:
        """Transform the payload to a string to be inserted in the message"""
        raise NotImplementedError()

class MqttSnBase(VaProto):
    """Base class for MQTT-SN-based protocols. Not to be used directly"""

    def __init__(self):
        self.client = mqttsn.client.Client(MQTTSN_SERVER_IP, MQTTSN_SERVER_PORT)
        self.client.connect()

    def send_message(self, path: str, dest: str, data: SendData):
        self.client.publish(path + "/" + dest, self.transform_payload(data))
    
    def transform_payload(self, data: SendData) -> bytes:
        raise NotImplementedError()

class CoApBase(VaProto):
    """Base class for CoAP-based protocols. Not to be used directly"""

    def __init__(self):
        self.client = asyncio.get_event_loop().run_until_complete(aiocoap.Context.create_client_context())
    
    def send_message(self, path: str, dest: str, data: SendData):
        request = aiocoap.Message(code=aiocoap.POST, uri=COAP_SERVER_IP + '/' + path, payload=self.transform_payload(data))
        asyncio.get_event_loop().run_until_complete(self.client.request(request))    
    
    def transform_payload(self, data: SendData) -> bytes:
        raise NotImplementedError()

# Protocols themselves

## MQTT
class MqttFlexbuf(MqttBase):
    def transform_payload(self, data: SendData) -> bytes:
        return encapsulate_bin(data, lambda d: flexbuffers.Dump(d))

class MqttMapBuf(MqttBase):
    def transform_payload(self, data: SendData) -> bytes:
        return encapsulate_bin(data, lambda d: MapBuffer(d).tobytes())

class MqttMessagePack(MqttBase):
    def transform_payload(self, data: SendData) -> bytes:
        return encapsulate_bin(data, lambda d: msgpack.packb(d))

## MQTT-SN
class MqttSnFlexbuf(MqttSnBase):
    def transform_payload(self, data: SendData) -> bytes: 
        return encapsulate_bin(data, lambda d: flexbuffers.Dumps(data))

class MqttSnMapBuf(MqttSnBase):
    def transform_payload(self, data: SendData) -> bytes:
        return encapsulate_bin(data, lambda d: MapBuffer(d).tobytes())

class MqttSnMessagePack(MqttSnBase):
    def transform_payload(self, data: SendData) -> bytes:
        return encapsulate_bin(data, lambda d: msgpack.packb(d))

## CoAP
class CoApFlexbuf(VaProto):
    def transform_payload(self, data: SendData) -> bytes:
        return encapsulate_bin(data, lambda d: flexbuffers.Dump(d))

class CoApMapBuf(VaProto):
    def transform_payload(self, data: SendData) -> bytes:
        return encapsulate_bin(data, lambda d: MapBuffer(d).tobytes())

class CoApMessagePack(VaProto):
    def transform_payload(self, data: SendData) -> bytes:
        return encapsulate_bin(data, lambda d: msgpack.packb(d))

## Others
class Hermes(VaProto):
    """ Implementation of Hermes, which is Mqtt + JSON. They sometimes just send
        a binary payload because JSON is not compatible with binary."""
    def __init__(self):
        self.client = mqtt.Client()

        self.client.connect(MQTT_SERVER_IP, MQTT_SEVER_PORT)

    def send_message(self, path: str, dest: str, data: SendData):
        msg = self.client.publish(self.transform_dest(path, dest), paylooad=self.transform_payload(data))
        msg.wait_for_publish()

    @staticmethod
    def transform_payload(data: SendData) -> bytes:
        def binary_or(data: SendData, f: Callable[[Dict[str, Any]], bytes]) -> bytes:
            if isinstance(data, dict):
                return f(data)
            return data

        return json.dumps(data).encode()
    
    @staticmethod
    def transform_dest(path:str, dest: str) -> str:
        if path.endswith("voice/"):
            return f"/hermes/audioServer/{dest}/audioFrame"

        if path.endswith("sound/"):
            return f"hermes/audioServer/{dest}/playBytes/123456789"

        return '/hermes' + path[:len('/asis_api')]  + '/' + dest

        

class HiveMind(VaProto):
    def __init__(self):
        key = "super_secret_access_key"
        crypto_key = "ivf1NQSkQNogWYyr"

        self.client = HiveMessageBusClient(key, crypto_key=crypto_key, ssl=False)

    def send_message(self, path: str, dest: str, data: SendData):
        mycroft_msg = mycroft.Message(dest, data)
        self.client.emit_mycroft(mycroft_msg)


all_protos = [
    MqttFlexbuf, MqttMapBuf, MqttMessagePack,
    MqttSnFlexbuf, MqttSnMapBuf, MqttSnMessagePack, 
    CoApFlexbuf, CoApMapBuf, CoApMessagePack,
    Hermes, HiveMind
]

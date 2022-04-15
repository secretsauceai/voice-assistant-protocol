from typing import Any, Callable, Dict, Protocol, Union
import asyncio
import logging

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


MQTT_SERVER_IP = "127.0.01"
MQTT_SEVER_PORT = 1883

MQTTSN_SERVER_IP = bytes("127.0.0.1",'utf-8')
MQTTSN_SERVER_PORT = 1883

COAP_SERVER_IP = "127.0.0.1"
COAP_SERVER_PORT = 5683

SendData = Union[Dict[str, Any],  bytes]
logging.getLogger("coap-server").setLevel(logging.ERROR)
logging.disable()


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
        msg = self.client.publish(path + "/" + dest, payload=self.transform_payload(data))
        msg.wait_for_publish()

    def transform_payload(self, data: SendData) -> bytes:
        """Transform the payload to a string to be inserted in the message"""
        raise NotImplementedError()

class MqttSnBase(VaProto):
    """Base class for MQTT-SN-based protocols. Not to be used directly"""

    def __init__(self):
        self.client = mqttsn.client.Client(MQTTSN_SERVER_IP, port=MQTTSN_SERVER_PORT)
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
        request = aiocoap.Message(code=aiocoap.POST, uri='coap://' + COAP_SERVER_IP + '/' + path, payload=self.transform_payload(data))
        asyncio.get_event_loop().run_until_complete(self.client.request(request).response)
    
    def transform_payload(self, data: SendData) -> bytes:
        raise NotImplementedError()

# Message formats
class FlexBufFormat:
    @staticmethod
    def transform_payload(data: SendData) -> bytes:
        return encapsulate_bin(data, lambda d: flexbuffers.Dumps(d))

class MapBufferFormat:
    @staticmethod
    def transform_payload(data: SendData) -> bytes:
        def indexKeys(d: Dict[str, Any]) -> Dict[int, Any]:
            def mapToInt(k: str) -> int:
                return {
                    "voice": 0,
                    "input": 1,
                }[k]

            return {mapToInt(k): v for k, v in d.items()}

        return encapsulate_bin(data, lambda d: MapBuffer(indexKeys(d)).tobytes())

class MsgPackFormat:
    @staticmethod
    def transform_payload(data: SendData) -> bytes:
        return encapsulate_bin(data, lambda d: msgpack.packb(d))
# Protocols themselves

## MQTT
class MqttFlexbuf(MqttBase):
    def transform_payload(self, data: SendData) -> bytes:
        return FlexBufFormat.transform_payload(data)

class MqttMapBuffer(MqttBase):
    def transform_payload(self, data: SendData) -> bytes:
        return MapBufferFormat.transform_payload(data)

class MqttMessagePack(MqttBase):
    def transform_payload(self, data: SendData) -> bytes:
        return MsgPackFormat.transform_payload(data)

## MQTT-SN
class MqttSnFlexbuf(MqttSnBase):
    def transform_payload(self, data: SendData) -> bytes: 
        return FlexBufFormat.transform_payload(data)

class MqttSnMapBuffer(MqttSnBase):
    def transform_payload(self, data: SendData) -> bytes:
        return MapBufferFormat.transform_payload(data)

class MqttSnMessagePack(MqttSnBase):
    def transform_payload(self, data: SendData) -> bytes:
        return MsgPackFormat.transform_payload(data)

## CoAP
class CoApFlexbuf(CoApBase):
    def transform_payload(self, data: SendData) -> bytes:
        return FlexBufFormat.transform_payload(data)

class CoApMapBuffer(CoApBase):
    def transform_payload(self, data: SendData) -> bytes:
        return MapBufferFormat.transform_payload(data)

class CoApMessagePack(CoApBase):
    def transform_payload(self, data: SendData) -> bytes:
        return MsgPackFormat.transform_payload(data)

## Others
class Hermes(VaProto):
    """ Implementation of Hermes, which is Mqtt + JSON. They sometimes just send
        a binary payload because JSON is not compatible with binary."""
    def __init__(self):
        self.client = mqtt.Client()

        self.client.connect(MQTT_SERVER_IP, MQTT_SEVER_PORT)

    def send_message(self, path: str, dest: str, data: SendData):
        msg = self.client.publish(self.transform_dest(path, dest), payload=self.transform_payload(data))
        msg.wait_for_publish()

    @staticmethod
    def transform_payload(data: SendData) -> bytes:
        def binary_or(data: SendData, f: Callable[[Dict[str, Any]], bytes]) -> bytes:
            if isinstance(data, dict):
                return f(data)
            return data

        return binary_or(data, lambda d: json.dumps(d).encode())
    
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
        self.client.run_forever()

    def send_message(self, path: str, dest: str, data: SendData):
        mycroft_msg = mycroft.Message(dest, data)
        self.client.emit_mycroft(mycroft_msg)


all_protos = [
    MqttFlexbuf, MqttMapBuffer, MqttMessagePack,
    #MqttSnFlexbuf, MqttSnMapBuffer, MqttSnMessagePack, 
    CoApFlexbuf, CoApMapBuffer, CoApMessagePack,
    Hermes#, HiveMind
]

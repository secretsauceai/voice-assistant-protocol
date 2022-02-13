#!/usr/bin/env python3

# Minimum python 3.6

import asyncio

import aiocoap
import aiocoap.resource as resource
import msgpack

registry_address = ""

def list_caps(payload):
    return ','.join([cap["name"] for cap in payload["request"]["capabilities"]])

class VapRequestResource(resource.Resource):
    def render_post(self, request):
        payload = msgpack.unpackb(request.payload)

        caps = list_caps(payload)
        print(f"Received a request with capabilities: {caps}")

        if payload["request"]["intent"] == "hello":
            data_response = {"capabilities": [{"text": "hello there"}]}
            return aiocoap.Message(payload=msgpack.packb(data_response))
            

class VapCanYouAnswerResource(resource.Resource):
    def render_get(self, request):
        payload = msgpack.unpackb(request.payload)
        caps = list_caps(payload)

        new_payload = {
            "confidence": 1.0
        }

        return aiocoap.Message(code=aiocoap.CHANGED, payload=msgpack.packb(new_payload))

class VapClient():
    async def _init(self):
        self.context = await aiocoap.Context.create_client_context()
    
    async def init(self):
        payload = {
            "name": "My test skill",
            "id": "com.example.test",
            "uniqueAuthenticationToken": "",
        }

        request = aiocoap.Message(code=aiocoap.GET, payload=msgpack.packb(payload), uri=f'coap://{registry_address}/vap/skillRegistry/connect')

        response = await self.context.request(request).response

        if response.code != aiocoap.CREATED:
            raise Exception(f"Failed to register skill: {response.code}")
        
        resp_payload = msgpack.unpackb(response.payload)
        print(f"Languages available: {','.join(resp_payload['languages'])}")

    async def registerUtts(self):
        payload = {
            "nluData": [
                {
                    "language": "en-US",
                    "intents": [
                        {
                            "name": "hello",
                            "utterances": [
                                {"text":"hello there"},
                                {"text":"hi"},
                            ]
                        }
                    ]
                },
                {
                    "language": "es-es",
                    "intents": [
                        {
                            "name": "hello",
                            "utterances": [
                                {"text":"hola, soy dora!"},
                                {"text":"hola"}
                            ]
                        }
                    ]
                }
            ]
        }
        
        request = aiocoap.Message(code=aiocoap.GET, payload=msgpack.packb(payload), uri=f'coap://{registry_address}/vap/skillRegistry/registerUtts')

        response = await self.context.request(request).response

        if response.code != aiocoap.CREATED:
            raise Exception(f"Failed to register utterances: {response.code}")

    async def close(self):
        payload = {"skillId":"com.example.test"}
        request = aiocoap.Message(code=aiocoap.GET, payload=msgpack.packb(payload), uri=f'coap://{registry_address}/vap/skillRegistry/skillClose')

        response = await self.context.request(request).response

        if response.code != aiocoap.DELETED:
            raise Exception(f"Failed to register utterances: {response.code}")



    
client = VapClient()

async def main():
    """Main entry point for the skill."""

    await client._init()

    await client.init()
    await client.registerUtts()

    
    root = resource.Site()
    root.add_resource((['vap','request']), VapRequestResource())
    root.add_resource((['vap','canYouAnswer']), VapCanYouAnswerResource())

    await aiocoap.Context.create_server_context(root)

    # Run forever
    await asyncio.get_running_loop().create_future()

    # If by wathever reason we have to shutdown call this
    await client.close()


if __name__ == "__main__":
    asyncio.run(main())
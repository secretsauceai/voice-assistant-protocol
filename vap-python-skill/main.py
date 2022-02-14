#!/usr/bin/env python3

# Minimum python 3.6

import asyncio

import aiocoap
import aiocoap.resource as resource
import msgpack

registry_address = "127.0.01"

def list_caps(payload):
    """Transform a list of capabilities into a string."""
    return ','.join([cap["name"] for cap in payload["request"]["capabilities"]])

class VapRequestResource(resource.Resource):
    """A CoAP resource that answers a VAP "request". When someone calls
    coap://<skill_address>/vap/request, this will trigger """

    def render_post(self, request):
        """Handle a POST message, VAP requests are always POSTs."""

        # Decode the MsgPack payload
        payload = msgpack.unpackb(request.payload)

        caps = list_caps(payload)
        print(f"Received a request with capabilities: {caps}")

        # Check if it is a known intent
        if payload["request"]["intent"] == "hello":
            data_response = {"capabilities": [{"name":"text", "text": "hello there"}]}

            # Message to be sent back to the server
            return aiocoap.Message(payload=msgpack.packb(data_response))
            

class VapCanYouAnswerResource(resource.Resource):
    """A CoAP resource that answers a VAP "can you answer". When someone calls
    coap://<skill_address>/vap/canYouAnswer, this will trigger"""

    def render_get(self, request):
        """Handle a GET message, VAP can you answer requests are always GETs."""

        payload = msgpack.unpackb(request.payload)
        caps = list_caps(payload)

        new_payload = {
            "confidence": 1.0
        }

        # Message to be sent back to the server
        return aiocoap.Message(code=aiocoap.CHANGED, payload=msgpack.packb(new_payload))

class VapClient():
    async def _init(self):
        """ Setup the client itself.
            Note: This is not __init__, we can't do this in _init because it is async,
            this needs to be called afterwards. """

        self.client = await aiocoap.Context.create_client_context()
    
    async def init(self):
        # Register whithin the server

        payload = {
            "name": "My test skill",
            "id": "com.example.test",
            "uniqueAuthenticationToken": "",
        }

        # Create message
        request = aiocoap.Message(code=aiocoap.GET, payload=msgpack.packb(payload), uri=f'coap://{registry_address}/vap/skillRegistry/connect')

        # Send it to the registry and wait for a response
        response = await self.client.request(request).response

        # Check if no error happened
        if response.code != aiocoap.CREATED:
            raise Exception(f"Failed to register skill: {response.code}")
        
        resp_payload = msgpack.unpackb(response.payload)
        print(f"Languages available: {','.join(resp_payload['languages'])}")

    async def registerIntents(self):
        # Send our utterances to the server for them to be taken account of

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
        
        # Create message
        request = aiocoap.Message(code=aiocoap.GET, payload=msgpack.packb(payload), uri=f'coap://{registry_address}/vap/skillRegistry/registerIntents')

        response = await self.client.request(request).response

        if response.code != aiocoap.CREATED:
            raise Exception(f"Failed to register utterances: {response.code}")

    async def close(self):
        # We have finished, let it know to the server

        payload = {"skillId":"com.example.test"}
        request = aiocoap.Message(code=aiocoap.GET, payload=msgpack.packb(payload), uri=f'coap://{registry_address}/vap/skillRegistry/skillClose')

        # Send it  to the registry and wait for a response
        response = await self.client.request(request).response

        # Check if no error happened
        if response.code != aiocoap.DELETED:
            raise Exception(f"Failed to disconenct from registry: {response.code}")

    async def notification(self):
        """ Some request started by the skill, right now, the protocol defines
        no response. """

        payload = {
            "data":[{
                "clientId": "123456789a",
                "capabilities": [{
                    "name": "text",
                    "text": "hello there"
                }]
            }]
        }

        # Create request
        request = aiocoap.Message(code=aiocoap.GET, payload=msgpack.packb(payload), uri=f'coap://{registry_address}/vap/skillRegistry/notification')

        # Send request
        await self.client.request(request)

    async def query(self):
        # Ask something to the system about a certain client or about the system itself.

        # Note: That inside "capabilities", except for "name", everything else 
        # is dependent on the capability and it is defined by it.
        payload = {
            "data":[{
                "clientId": "123456789a",
                "capabilities": [{
                    "name": "preferences",
                    "what": "color"
                }]
            }]
        }

        # Create request
        request = aiocoap.Message(code=aiocoap.GET, payload=msgpack.packb(payload), uri=f'coap://{registry_address}/vap/skillRegistry/notification')

        response = await self.client.request(request).response

        # Find the same capability, preferences, that we sent, remember we can 
        # receive multiple capabilities and multiple clients in a same response.
        # We find it by applying a filter
        cap_color = list(filter(
            lambda c: c["name"]=="preferences",
            msgpack.unpackb(response)["data"][0]["capabilities"]))

        # Now that we have a list, get the first item and return the color that
        # we asked for
        color = cap_color[0]["color"]

        print(f"Preferred color: {color}")
    
client = VapClient()

async def main():
    """Main entry point for the skill."""

    # CoAP client initialization
    await client._init()

    # Connect to the skill registry and send utterances and 
    await client.init()
    await client.registerIntents()

    # Register the CoAP paths that we can answer
    root = resource.Site()
    root.add_resource((['vap','request']), VapRequestResource())
    root.add_resource((['vap','canYouAnswer']), VapCanYouAnswerResource())

    # Start the server
    await aiocoap.Context.create_server_context(root)

    # Perform notifications and queries, note this can be done whenever
    await client.notification()
    await client.query()

    # Run forever
    await asyncio.get_running_loop().create_future()

    # If by wathever reason we have to shutdown call this
    await client.close()


if __name__ == "__main__":
    asyncio.run(main())
#!/usr/bin/env python3

# Minimum python 3.6

import asyncio
import re
from typing import Optional

import aiocoap
import msgpack

registry_address = "127.0.0.1"
all_coaps_ip4 = "224.0.1.187"
skill_id = "com.example.test"
coap_host_reg = re.compile(r'^coap:\/\/([1-9.a-zA-Z]+)\/')

def list_caps(payload):
    """Transform a list of capabilities into a string."""
    return ','.join([cap["name"] for cap in payload["request"]["capabilities"]])

class VapClient():
    async def _init(self):
        """ Setup the client itself.
            Note: This is not __init__, we can't do this in _init because it is async,
            this needs to be called afterwards. """

        self.client = await aiocoap.Context.create_client_context()

    async def __find_registry(self) -> Optional[str]:
        """ Find a registry using CoAP's discovery """
        request = aiocoap.Message(code=aiocoap.GET, uri=f'coap://{all_coaps_ip4}/.well-known/core?rt=vap-skill-registry')
        response = await self.client.request(request).response

        if "vap-skill-registry" in response.payload:
            m = coap_host_reg.match(response.payload.decode())

            if m:
                return request.get_request_uri(m.group(1))
        
        return None
    
    
    async def init(self):
        # Register whithin the server

        payload = {
            "name": "My test skill",
            "id": skill_id,
            "vapVersion": "1.0.0",
            "uniqueAuthenticationToken": "",
        }

        # Create message
        request = aiocoap.Message(code=aiocoap.POST, payload=msgpack.packb(payload), uri=f'coap://{registry_address}/vap/skillRegistry/connect')

        # Send it to the registry and wait for a response
        response = await self.client.request(request).response

        # Check if no error happened
        if response.code != aiocoap.CREATED:
            raise Exception(f"Failed to register skill: {response.code}")
        
        resp_payload = msgpack.unpackb(response.payload)
        langs_id = 0
        def lang_to_str(lang):
            if not lang[0] is None:
                first_phase = f'{lang[1]}-{lang[0]}'
            else:
                first_phase = lang[1]
            
            if not lang[2] is None:
                return f'{first_phase}-{lang[2]}'
            else:
                return first_phase

        print(f"Languages available: {','.join( [lang_to_str(x) for x in resp_payload[langs_id]])}")

    async def registerIntents(self):
        # Send our utterances to the server for them to be taken account of

        payload = {
            "skillId": skill_id,
            "nluData": [
                {
                    "language": {
                        "language": "en",
                        "country": "us",
                        "extra": None
                    },
                    "intents": [
                        {
                            "name": "hello",
                            "utterances": [
                                {"text": "hello there"},
                                {"text": "hi"},
                            ],
                            "slots": []
                        }
                    ],
                    "entities": []
                },
                {
                    "language": {
                        "language": "en",
                        "country": "es",
                        "extra": None
                    },
                    "intents": [
                        {
                            "name": "hello",
                            "utterances": [
                                {"text": "hola, soy dora!"},
                                {"text": "hola"}
                            ],
                            "slots": []
                        }
                    ],
                    "entities": []
                }
            ]
        }
        
        # Create message
        request = aiocoap.Message(code=aiocoap.POST, payload=msgpack.packb(payload), uri=f'coap://{registry_address}/vap/skillRegistry/registerIntents')

        response = await self.client.request(request).response

        if response.code != aiocoap.CREATED:
            raise Exception(f"Failed to register utterances: {response.code}")

    async def close(self):
        # We have finished, let it know to the server

        payload = {"skillId": skill_id}
        request = aiocoap.Message(code=aiocoap.DELETE, payload=msgpack.packb(payload), uri=f'coap://{registry_address}/vap/skillRegistry/skills/{skill_id}')

        # Send it  to the registry and wait for a response
        response = await self.client.request(request).response

        # Check if no error happened
        if response.code != aiocoap.DELETED:
            raise Exception(f"Failed to disconenct from registry: {response.code}")

    async def notification(self):
        """ Some request started by the skill, right now, the protocol defines
        no response. """

        payload = {
            "skillId": skill_id,
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
        await self.client.request(request).response

    async def query(self):
        # Ask something to the system about a certain client or about the system itself.

        # Note: That inside "capabilities", except for "name", everything else 
        # is dependent on the capability and it is defined by it.
        payload = {
            "skillId": skill_id,
            "data":[{
                "clientId": "123456789a",
                "capabilities": [{
                    "name": "preferences",
                    "what": "color"
                }]
            }]
        }

        # Create request
        request = aiocoap.Message(code=aiocoap.GET, payload=msgpack.packb(payload), uri=f'coap://{registry_address}/vap/skillRegistry/query')

        response = await self.client.request(request).response

        # Check if no error happened
        if response.code != aiocoap.CONTENT:
            raise Exception(f"Failed to disconenct from registry: {response.code}")

        data_id = 0
        capabilities_id = 1

        # Find the same capability, preferences, that we sent, remember we can 
        # receive multiple capabilities and multiple clients in a same response.
        # We find it by applying a filter
        cap_color = list(filter(
            lambda c: c["name"]=="preferences",
            msgpack.unpackb(response.payload)[data_id][0][capabilities_id]))

        # Now that we have a list, get the first item and return the color that
        # we asked for
        color = cap_color[0]["color"]

        print(f"Preferred color: {color}")

    async def register(self):
        skill_id = "test_skill"

        message = aiocoap.Message(
            code=aiocoap.GET,
            observe=9999999999999,
            uri=f'coap://{registry_address}/vap/skillRegistry/skills/{skill_id}'
        )

        request = self.client.request(message)
        

        async for r in request.observation:
            print("Got request from registry: ")
            payload = msgpack.unpackb(r.payload)
            print(payload)
            request_id = 1
            type_id = 0
            request_type = payload[request_id][type_id]
            if request_type == "canYouAnswer":
                print("Got a canYouAnswer request")
            

            
        

    
client = VapClient()

async def main():
    """Main entry point for the skill."""

    # CoAP client initialization
    await client._init()

    # Connect to the skill registry and send utterances and 
    await client.init()
    await client.registerIntents()

    # Perform notifications and queries, note this can be done whenever
    await client.notification()
    await client.query()

    await client.register()

    # Run forever
    await asyncio.get_running_loop().create_future()

    # If by wathever reason we have to shutdown call this
    await client.close()


if __name__ == "__main__":
    asyncio.run(main())
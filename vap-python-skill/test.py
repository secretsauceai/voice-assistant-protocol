import logging
import asyncio
from pyparsing import Regex
import re

from aiocoap import *

logging.basicConfig(level=logging.INFO)

async def main():
    protocol = await Context.create_client_context()

    request = Message(code=GET, uri='coap://127.0.0.1/.well-known/core', mtype=NON)

    try:
        response = await protocol.request(request).response
    except Exception as e:
        print('Failed to fetch resource:')
        print(e)
    else:
        print('Code: %s\nPayload: %r\nFrom: %s'%(response.code, response.payload, response.get_request_uri()))
        reg = re.compile(r'^coap:\/\/([0-9.a-zA-Z]+)\/')
        reg_vap_uri = re.compile(r'^<\/((?:[a-zA-Z]+)+)\/?>;rt="vap-skill-registry"$')

        host = reg.match(response.get_request_uri()).group(1)
        vap_uri = reg_vap_uri.match(response.payload.decode('utf-8')).group(1)
        print(f"coap://{host}/{vap_uri}/")

        

if __name__ == "__main__":
    asyncio.run(main())
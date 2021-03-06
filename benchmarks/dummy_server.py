import asyncio
import logging

import aiocoap.resource as resource
import aiocoap


class MirrorResource(resource.Resource):
    """Example resource which supports the GET and PUT methods. It sends large
    responses, which trigger blockwise transfer."""

    async def render_get(self, request):
        return aiocoap.Message(code=aiocoap.CONTENT, payload=b'</test>;rt="test";ct=0')

    async def render_put(self, request):
        print('PUT payload: %s' % request.payload)
        self.set_content(request.payload)
        return aiocoap.Message(code=aiocoap.CHANGED, payload=self.content)
    
    async def render_post(self, request):
        print('POST payload: %s' % request.payload)
        self.set_content(request.payload)
        return aiocoap.Message(code=aiocoap.CHANGED, payload=self.content)

# logging setup

logging.basicConfig(level=logging.ERROR)
logging.getLogger("coap-server").setLevel(logging.ERROR)
logging.disable()
async def main():
    # Resource tree creation
    root = resource.Site()

    res = MirrorResource()
    root.add_resource(['asis_api', 'voice'], res)
    root.add_resource(['asis_api', 'sound'], res)
    root.add_resource(['asis_api', 'intent', 'request_weather'], res)
    root.add_resource(['.well-known', 'core'], res)

    await aiocoap.Context.create_server_context(
        root, multicast=['wlp5s0'])

    # Run forever
    await asyncio.get_running_loop().create_future()

if __name__ == "__main__":
    asyncio.run(main())

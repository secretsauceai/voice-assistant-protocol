# Python skill example

`main.py` is an example of a skill. It is using directly `aiocoap` and `msgpack` libraries, so it is pretty low-level. A library can be made for having higher-level code.

Implements all paths, though does not take much care for errors and edge cases, also, it doesn't implement discovery, which would let this find a registry without any hard coded IP and with little user intervention (a user might only need to select which of the available registries, if more than one to connect to).

## Behavior

First, it will connect to the registry (inside `registry_address`), once accepted, it will send it's intents and utterances to the server, at this point the registration is complete. After that it will send a notification to a client and query the system for users favorite color.Finally, it will wait for any event the registry might send.

Note: Take into that account that no capability has been defined as of now, and everything here is just for showing the features.
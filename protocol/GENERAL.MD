Capabilities are meant to be used by all parts of a voice assistant:

* Client: To send information and perform actions
* Server:  To transform and limit ( in any direction) information, 
            and answer to information requests and server-related actions
* Skills: To receive information, ask for actions to be done and request information


## Confirmability

CoAP calls messages that must have their arrival confirmed as confirmable,
some paths have are "Confirmable: Mandatory", otherwise they'll have 
"Confirmable: Optional".

## Architecture

All parts of the structure (registries, clients, skills) send and receive messages
and as so, all of them need to implement both a client and a server. Clients and skills can expect that the only one connecting to them is the server (though be aware of attackers if they are accesible by internet), in the case of skills, the server can send multiple concurrent requests, the skill is free to answer them in any order.

### Grouping

If a server receives many notifications request from several skills, and they
target the same client, the server is free to group them by clients and send one
notification to the client with capabilities from multiple skills.

## Errors

Error structure:
* code: int -> Same CoAP code is in return, this might make things easier for the receiver.
* type: String -> An string with an identifier of what happened.
* object: Optional\<String> -> What cause the error, not every error has one.
* docRef: Optional\<String> -> Some online documentation about the error.


If a client/skill (or a server from them) requires something that is not found:

**Answer** (Code: 404 Not found)
* code = 404
* type = "not found"
* object -> Path that was not found


If the received data is not valid MsgPack/CBOR:

**Answer** (Code: 400 Bad request)
* code = 400
* type = "malformed content"


If the message lacks a mandatory field:

**Answer** (Code: 408 Request Entity Incomplete)
* code = 408
* type = "missing field"
* object -> The field that was lacking


If the message uses a method not registered by the specification (using a put instead of a POST) an implementation CAN return a 405 "Method not allowed".

If any content format other than "#!TODO: Choose content format code#!":
**Answer** (Code: 415 Unsupported Content-Format)


If an exception/unrecoverable error arise while processing a petition the answer will be 500, if a skill sends a 500 as an answer, the client shall receive a 500 too along with some answer indicating of an error happening.

# IP Addresses

IP addresses should never be tracked, as they could change due to a network change, or in the case of registries a system might employ several registry nodes, and the one you send the connect request might not be the one the system assigned the skill/client to.
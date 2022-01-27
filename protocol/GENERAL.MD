Capabilities are meant to be used by all parts of a voice assistant:

* Client: To send information and perform actions
* Server:  To transform and limit ( in any direction) information, 
            and answer to information requests and server-related actions
* Skills: To receive information, ask for actions to be done and request information


## Confirmability

CoAP calls messages that must have their arrival confirmed as confirmable,
some paths have are "Confirmable: Mandatory", otherwise they'll have 
"Confirmable: Optional".

## Errors

#!TODO: Content-format for deflate CBOR 11060#!

Error structure:
* code: int -> Same CoAP code is in return, this might make things easier for the receiver.
* type: String -> An string with an identifier of what happened.
* object: Optional\<String> -> What cause the error, not every error has one.


If a client/skill requires something that is not found:

**Answer** (Code: 404 Not found)
* code = 404
* type = "not found"
* object -> Path that was not found


If the received data is not valid MsgPack/CBOR:

**Answer** (Code: 400 Bad request)
* code = 400
* type = ""

#!TODO#! The server might return some info about the error. include this


If the message lacks a mandatory field:

**Answer** (Code: 408 Request Entity Incomplete)
* code = 408
* type = "missing field"
* object -> The field that was lacking


If the message uses a method not registered by the specification (using a put instead of a POST) an implementation CAN return a 405 "Method not allowed". #!TODO: This is related to the TODO of not making use of methods#!

#!TODO: Should be a 413 "Payload too large" be considered?#!


If any content format other than "#!TODO: Choose content format code#!":
**Answer** (Code: 415 Unsupported Content-Format)
#!TODO#! Is anything even needed here?


If an exception/unrecoverable error arise while processing a petition the answer will be 500, if a skill sends a 500 as an answer, the client shall receive a 500 too (#!TODO: Does this make sense?#!) along with some answer indicating of an error happening.
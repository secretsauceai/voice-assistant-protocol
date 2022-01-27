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

#!TODO: Should error descriptions be standard?#!
#!TODO: Content-format for deflate CBOR 11060#!

If a client/skill requires something that is not found the return code would be 404.

If the received data is not valid MsgPack/CBOR a 400 "Bad Request" error is returned. The server might return some info about the error. 

If the message lacks a mandatory field a 408 "Request Entity Incomplete" error is returned.

If a request has an invalid (Connection or Unique) AuthenticationToken then a 401 "Unauthorized error" is returned.

If the message uses a method not registered by the specification (using a put instead of a POST) an implementation CAN return a 405 "Method not allowed". #!TODO: This is related to the TODO of not making use of methods#!

#!TODO: Should be a 413 "Payload too large" be considered?#!


If any content format other than "#!TODO: Choose content format code#!" the answer should be 415 "Unsupported Content-Format"

If an exception/unrecoverable error arise while processing a petition the answer will be 500, if a skill sends a 500 as an answer, the client shall receive a 500 too (#!TODO: Does this make sense?#!) along with some answer indicating of an error happening.

Any answer with a wrong ConnectionAuthenticationToken can be ignored.
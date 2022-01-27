

# TODOS

* Internally tagged vs externally tagged
* Should capabilities be written as organization.name (like vap.dyn_nlu or lily.custom_capability)?
* How to resolve a skill that changes server? Maybe we can use the zeroconf name? What alternative do we have if zeroconf is optional?
* Think on addresses of registries, if making them not to change would be an improvement on security, if so by how much?.
* TCP or UDP?
* Given that any path does not implement two methods, how about they are ignored?
* Instead of confirmable, can we just wait for the answer?
* Should session data be preferably non-confirmable?
* Is UniqueAuthenticationTokenRequired on the clients?
* Context data should be handled here?

## To resolve by myself

* Some sort of recovery mechanism in case the client "forgets" their UniqueAuthorizationSkill or it becomes corrupted?
* What happens if a single address registers multiple IDs from other clients, therefore preventing them to be connected?
* Can we detect that a ConnectionAuthorizationToken or a GlobalAuthorizationToken has been compromised? Should we include any mechanism to change it? (Note: Even if we can detect that, maybe we don't need an explicit mechanism, but rather just closing down the connection and starting it again, how would this affect users?).
* A session_start rejected because of two simulatenous activations should be considered an error?
* ASR, NLU and Skills might take a while, should we send a "temporal answer" to a session start.
* A client needs to know about languages in the server? Should it be able to choose a prefered one if there are multiple? I think yes.

# Location:

Clients will search for the client registry using zeroconf or CoAP 

Name of the Zeroconf service for the Client Registry "vap-client-registry.udp"

# Connection

## Init:

*POST* **Server/vap/client_registry/connect** (Confirmable: Mandatory, Client -> Registry)
* Name:
* Id: (like org.company.product)
* VAP Version:
* UniqueAuthorizationToken:

Client connects with client register. We have a uniqueAuthorizationToken, this let us know and understand that the client has been allowed in the system, and that it is who's saying to be. This is required mostly because of the possibility of user authenticaion, if any client was allowed to access the server, aunthencation would be useless, someone not allowed in the assistant could just attack the system with a hacked client.

The UniqueAuthorizationToken works just like the client one, the first time a client connects, that field is empty here and the server will asign one, which the client needs to send the next time they connect to the system. Failing this means that the client won't be allowed into the system.

**Answer**
* One of:
    * OK! (Code: 201 Created)
        * UniqueAuthorizationToken: (Optional)
        * ConnectionAuthorizationToken
    * Errors:
        * Which?

The server will answer a UniqueAuthorizationToken only if this is the first time the client is connecting and we don't have any record of it.

It will also issue a ConnectionAuthorizationToken which has to be presented on each request, the server will present it too.

## Session start

*POST* **Server/vap/client_registry/session_start** (Confirmable: Optional, Client -> Registry)
* Optional capabilities:
    * Wakeword authentication (wake word is sent for double checking with a bigger model or to check that the user has permission for it to be in the system).
* ConnectionAuthorizationToken
* ExactTimeStamp:  #!TODO#! Is this really needed?

This signals that a

**Answer**
* One of:
    * OK (Code: 201 Created)!
        * ConnectionAuthorizationToken
    * Error:
        * User authentication (voice, face, wathever) wasn't successful
        * Wakeword double check wasn't succesful
        * Two or more clients were activated by the same user at the same time and another got the focus. (#!TODO#!Is this technically an error?)

*POST* **Server/vap/client_registry/session_data** (Confirmable: Optional, Client -> Registry)
* Capabilities data:
    * Capability:
    * Other fields
* Last fragment: bool

**Answer**
* Ok (Code: 231 Continue)
    * ConnectionAuthorizationToken
* If last fragment:
    * Ok (Code: #!TODO#!)
    * ConnectionAuthorizationToken
    * #!TODO#! Should we wait for the answer from the whole pipeline? Or rather send a "everything alright we are processing"? Note: An everything alright we are processing seems interesting from the viewpoint of a device that wants to show the user that there's processing being done.

Final answer:
* Capabilites data:
    * Capability:
    * Other fields
* Session end: bool

## On Anytime

*POST* **Client/vap/notification** (Confirmable: Optional, Registry -> Client)
* Capabilities data:
    * Capability:
    * Other fields

#!TODO#! Does this need an answer? An alternative could be to propose having it as confirmable.

## When closing

*POST* **Server/vap/client_registry/client_close** (Confirmable: Mandatory, Client -> Registry)
* ID: (The one like org.company.product)  
* ConnectionAuthorzationToken:

**Answer**:
* Either:
    * OK! (Code: #!TODO#!)
        * ConnectionAuthorzationToken:
    * Error:
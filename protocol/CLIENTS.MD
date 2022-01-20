

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

* Authentication? Of the client and the user using it
* How to deal with multiple devices waking up at the same time?

# Location:

Clients will search for the client registry using zeroconf or CoAP 

Name of the Zeroconf service for the Client Registry "vap-client-registry.udp"

# Connection

## Init:

*POST* **Server/vap/client_registry/connect** (Confirmable: Mandatory, Client -> Registry)
* Name:
* Id:
* VAP Version:

Client connects with client register.

**Answer**
* One of:
    * OK! (Code: 201 Created)


## Session start

*POST* **Server/vap/client_registry/session_start** (Confirmable: Optional, Client -> Registry)
* Optional capabilities:
    * Wakeword authentication (wake word is sent for double checking with a bigger model )

#!TODO!# Another look at this

**Answer**
* One of:
    * OK (Code: 201 Created)!
    * Error:
        * User authentication (voice, face, wathever) wasn't successful
        * Wakeword double check wasn't succesful

*POST* **Server/vap/client_registry/session_data** (Confirmable: Optional, Client -> Registry)
* 

#!TODO!# Finish this
#!TODO!# Also, how do we send the final answer?

## On Anytime

*POST* **Client/vap/notification** (Confirmable: Optional, Registry -> Client)
#!TODO!# Finish this

## When closing

*POST* **Server/vap/client_registry/client_close** (Confirmable: Mandatory, Client -> Registry)
#!TODO!# Finish this
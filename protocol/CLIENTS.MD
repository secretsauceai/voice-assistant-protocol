

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
* Think of ways the system can ask for the client to be accepted into the network (it could be rejected)
* Is timestamp really needed for simultaneous activation checking?

## To resolve by myself

* Some sort of recovery mechanism in case the client "forgets" their UniqueAuthorizationSkill or it becomes corrupted?
* What happens if a single address registers multiple IDs from other clients, therefore preventing them to be connected?
* Can we detect that a GlobalAuthorizationToken has been compromised? Should we include any mechanism to change it? (Note: Even if we can detect that, maybe we don't need an explicit mechanism, but rather just closing down the connection and starting it again, how would this affect users?).
* A session_start rejected because of two simulatenous activations should be considered an error?
* ASR, NLU and Skills might take a while, should we send a "temporal answer" to a session start.
* A client needs to know about languages in the server? Should it be able to choose a prefered one if there are multiple? I think yes.

# Location:

Clients will search for the client registry using zeroconf or CoAP 

Name of the Zeroconf service for the Client Registry "vap-client-registry.udp"

# Connection

## Init:

*POST* **Server/vap/clientRegistry/connect** (Confirmable: Mandatory, Client -> Registry)
* name:
* id: String -> (like org.company.product)
* vapVersion:
* uniqueAuthenticationToken: 

Client connects with client register. We have a uniqueAuthenticationToken, this let us know and understand that the client has been allowed in the system, and that it is who's saying to be. This is required mostly because of the possibility of user authenticaion, if any client was allowed to access the server, aunthencation would be useless, someone not allowed in the assistant could just attack the system with a hacked client.

The uniqueAuthenticationToken works just like the skills one, the first time a client connects, that field is empty here and the server will asign one, which the client needs to send the next time they connect to the system. Failing this means that the client won't be allowed into the system.

**Answer**
* One of:
    * OK! (Code: 201 Created)
        * UniqueAuthenticationToken: (Optional)
        * locales: Locale
    * Errors:
        * 400 Bad Request: Id already exists? (should Ids be unique?)
        * 400 Bad Request: vapVersion incompatible
        * 401 Unauthorized: uniqueAuthenticationToken wrong
            * code = 401
            * type = "uniqueAuthenticationTokenErronous"
        * 401 Unauthorized: connection denied by policy or by the user (maybe the user didn't accept the client or it is blocked)
            * code = 401
            * type = "connectionDenied"

The server will answer a UniqueAuthenticationToken only if this is the first time the client is connecting and we don't have any record of it.

## Session start

*POST* **Server/vap/clientRegistry/sessionStart** (Confirmable: Optional, Client -> Registry)
* capabilities: Optional<[]> ->
    * name: String
    * <capability data>
* exactTimeStamp:?

This signals that a client wants to start a session. At this point we can send capabilities too, they are meant for user authorization and wake word double checking (with a bigger, slower, more accurate model in the server). Of course, the server is free to either accept it or reject if because of any reason.

**Answer**
* OK (Code: 201 Created)!
* Error:
    * User authentication (voice, face, wathever) wasn't successful (is this an error?)
    * Wakeword double check wasn't succesful (is this an error?)
    * Two or more clients were activated by the same user at the same time and another got the focus. (#!TODO#!Is this technically an error?)
    * Too many clients? (#!TODO#!Is this technically an error?)

*POST* **Server/vap/clientRegistry/sessionData** (Confirmable: Optional, Client -> Registry)
* capabilities: [] ->
    * name: String
    * <capability data>
* lastFragment: bool

**Answer**
* Ok (Code: 231 Continue)
* If last fragment:
    * Ok (Code: 201 Created) #!TODO#! Should we allow for cacheable responses with 205 for those that the skill said it was cacheable?
    * #!TODO#! Should we wait for the answer from the whole pipeline? Or rather send a "everything alright we are processing"? Note: An everything alright we are processing seems interesting from the viewpoint of a device that wants to show the user that there's processing being done.

Final answer:
* capabilities: [] ->
    * name: String
    * <capability data>
* endSession: bool

## On Anytime

*POST* **Client/vap/notification** (Confirmable: Optional, Registry -> Client)
* capabilities: [] ->
    * name: String
    * from: String -> Skill name, can also be the system itself.
    * <capability data>

#!TODO#! Does this need an answer? An alternative could be to propose having it as confirmable.

## When closing

*POST* **Server/vap/clientRegistry/clientClose** (Confirmable: Mandatory, Client -> Registry)
* clientId: (The one like org.company.product)  

**Answer**:
* Either:
    * OK! (Code: 202 Deleted)
    * Error:
        * 400 Bad request: clientId does not exist
            * code: int -> 400
            * type: "wrong clientId"
            * object: String -> The clientId that didn't exist
        * 401 Unauthorized: This clientId is not related to this address #!TODO#! Should this be emptier to make DDOS attacks more difficult?
            * code: int -> 401
            * type: "unauthorized"
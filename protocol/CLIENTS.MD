# Location

Clients will search for the client registry using zeroconf or CoAP 

Name of the Zeroconf service for the Client Registry "vap-client-registry.udp"

# Connection

## Init

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
    * User authentication (voice, face, wathever) wasn't successful.
    * Wakeword double check wasn't succesful.
    * Two or more clients were activated by the same user at the same time and another got the focus.
    * Too many clients

*POST* **Server/vap/clientRegistry/sessionData** (Confirmable: Optional, Client -> Registry)
* capabilities: [] ->
    * name: String
    * <capability data>
* lastFragment: bool

**Answer**
* Ok (Code: 231 Continue)
* If last fragment: (Note: It may take a while)
    * Ok (Code: 201 Created)
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
        * 401 Unauthorized: This clientId is not related to this address
            * code: int -> 401
            * type: "unauthorized"
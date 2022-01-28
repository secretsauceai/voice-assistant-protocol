# TODOS

* Internally tagged vs externally tagged? (both Alexa and Hermes use internally tagged)
* Better name than VAP?
* Signals and skill queries on VAP?
    * Task polling to detect changes? (Maybe this can be done as a separate skill)
* Context data in request (and maybe in "can answer"), user input, confidence, slots ...
* Should capabilities be written as organization.name (like vap.dyn_nlu or lily.custom_capability)?
* How to resolve a skill that changes server? Maybe we can use the zeroconf name? What alternative do we have if zeroconf is optional?
* The capabilities of the client are needed for the canAnswer?
* Think on addresses of registries, if making them not to change would be an improvement on security, if so by how much?.
* TCP or UDP?
* Given that any path does not implement two methods, how about they are ignored?
* Should we join server queries and notifications using GET/POST into just one path? Queries would use GET and notifications POST.
* Instead of confirmable, can we just wait for the answer?
* How to treat versions? As Strings, as some combination of numbers...
* What is the type of the UniqueAuthentcationToken
* Maybe the uniqueAuthenticationToken can be adapted/related to keys in OSCORE/DTLS?
* Can we deal with differences in how a required slot is asked?
* Is every NLU capable of being strict (like in register utterances)?
* Modeled mostly after Alexa and Hermes, maybe remodel it like Google's scenes, more advanced?
* We need a "systemId" for the "core/server/voice assistant" to be referenced.
* Think of ways the system can ask for the skill to be accepted (it could be rejected)

## To resolve by myself

* Specify language negotiation strategy (first analyze them and then show possible answers). Available in fluent-langneg (https://docs.rs/fluent-langneg/latest/fluent_langneg/negotiate/index.html):
    * Exact match
    * Locale range
    * Use ICU LikelySubtags
    * Different variant of the same locale
    * Only consider language stripped and then maximize it
    * Different region same locale (I like this one, easy to implement and good results)
* CoAP discovery vs Zeroconf
* CBOR Vs MsgPack

## To check by others

"vap-skill-registry" service name

*Note:* The path shown on each message is the CoAP URL, Server means is a path the server is listening on while Skill is the same for any skill.


# Location

Skills will search for the skill registry using zeroconf a.k.a bonjour a.k.a mDNS/DNS-SD

Name of the Zeroconf service for the Skill Registry "vap-skill-registry.udp"


# Connection

## Init

*POST* **Server/vap/skillRegistry/connect** (Confirmable: Mandatory, Skill -> Registry)
* name: Human readable name of the skill
* id: Unique ascii based name of the skill in the form of org.company.product
* vapVersion:
* uniqueAuthenticationToken (Optional)

Skill connects with skill register. #!TODO#! What to do with different versions?

**Answer:**
* One of:
    * OK! (Code: 201 Created)
        * langs: \[languages\] -> Which languages are present in the system
        * uniqueAuthenticationToken (If not provided before)
    * Error:
        * 400 Bad Request: Id already exists? (should Ids be unique?)
        * 400 Bad Request: vapVersion incompatible
        * 401 Unauthorized: uniqueAuthenticationToken wrong
            * code = 401
            * type = "uniqueAuthenticationTokenErronous"
        * 401 Unauthorized: connection denied by policy or by the user (maybe the user didn't accept the client or it is blocked)
            * code = 401
            * type = "connectionDenied"


The authentication tokens serve so that both the skill registry and the skill know
that no identity theft is ocurring.

First, the UniqueAuthenticationToken. This makes sures that no skill tries to 
act as another one already registered (running now or not). This serves as
two purposes: first, if the server holds any information about the skill this
makes it difficult for a skill to access that information about another skill (
whether it's data, config, or just the cache of the compiled NLU model) and two,
this let us limit some capabilities to some skills, capabilities like accessing
the whole config or modifying parts of the system. When a skill first connects
to a system it will send the init message with no UniqueAuthenticationToken, then,
the server, when it receives the message if it is the first time that it sees
that id then it will generate a token which will be sent to the skill, that same
token has to be resent each time the skill wants to connect to a system.

*NOTE*: This means that a skill needs permanent storage

After that send:
*POST* **Server/vap/skillRegistry/registerUtts** (Confirmable: Mandatory, Skill -> Registry)
* nluData:[] -> One set per language
    * language: Language
    * intents: [] ->
        * name: String
        * utterances: [] ->
            * text: String
        * slots:  [] ->
            * entity: String -> Name of entity.
            * required: bool -> If true the assistant must ask for it beforehand.
            * prompt: Optional\<String> -> If required and not present the slot will be asked .with this.
            * reprompt: Optional\<String> -> If prompt failed try with this one.
    * entities: [] ->
        * strict: bool (default: false) -> If true only the words in the list will be accepted
        * data:
            * value: String
            * synonyms: \[String\]

**Answer:**
* One of:
    * Ok (Code: 201 Created)
    * Error (Code: 408 Request Entity Incomplete):
        * code: int = 408
        * type: String = "missing entity" | "missing slot"
        * object: String -> The entity that's missing


## Skill interactions:

*GET* **Skill/vap/canYouAnswer**  (Confirmable: Optional, Registry -> Skill)
* request:
    * type: "event"|"intent" -> Internally tagged request data.
    * capabilities: [] ->
        * name: String
        * \<capability data>

    * type: "event"
    * name: String

    * type: "intent"
    * name: String
    * locale: Language
    * input: String
    * slots: [] ->
        * name: String
        * value: Optional\<String>

The server is asking "Can you answer this request?"

**Answer:** (Code: 205 Content)
* confidence: float

 When multiple skills are capable of answering the same request (it could be a
 generic request like "turn off the kitchen lights") we send them the intent and the slots (like "turn_off" "kitchen lights") and each skill returns an 
 estimation of how well they can answer this request.

*POST* **Skill/vap/request** (Confirmable: Optional, Registry->Skill)
* client:
    * systemId: String -> An unique ID for the client to be recognized, it doesn't have to be the same one that the client provides, can be randomly generated.
    * capabilities: [] ->
        * name: String
        * version: int

* request:
    * type: "event"|"intent" -> Internally tagged request data.
    * capabilities: [] -> Extra data sent by the client
        * name: String
        * <capability data>

    * type: "event"
    * Name

    * type: "intent"
    * name: String
    * locale: Language
    * input: String
    * slots: [] ->
        * name: String
        * value: Optional\<String>

**Answer:** (Code: 204 Changed, a skill can return "205 Content" if no internal state changed and the same request will always return the same answer)
* capabilities:
    * name: String
    * <capability data>


## At any time:

*POST* **Server/vap/skillRegistry/notification** (Confirmable: Optional, Skill -> Registry)
* data: (Can send to multiple at the same time, one per client to send)
    * clientId: String -> SystemId of client 
    * capabilities:
        * name: String
        * <capability data>
    

 A notification is a message started by a skill. Sometimes a skill needs to send
 information on it's own, not as an answer to something the user asked, whether
 it's a stream of sound or whether is just an alarm. Note that the server can
 also receive notifications.

**TODO:**  Does this needs an answer? Most probably yes, otherwise they might not know if everything went ok.


*GET* **Server/vap/skillRegistry/query** (Confirmable: Optional, Skill -> Registry)
* data: [] -> (Can send to multiple at the same time, one per client to send)
    * clientId: String -> SystemId of client 
    * capabilities:
        * name: String
        * <capability data>
 
**Answer:** (Code: 205 Content)
 * data: [] ->
    * clientId: String -> SystemId of client 
    * capabilities:
        * name: String
        * code: int -> CoAP result code

        * code: 205 (Content)
        * <capability data>

        * code: 404 (Not found) -> Client/Capability not found
        * object: String -> What was not found
        * kind: "client"|"capability"

        * code: 408 (Request Entity Incomplete)
        * object: String -> What was lacking
        * type: Optional "int"|"string" .... -> The type of what is lacking
        * docRef: Optional\<String> -> Some reference to documentation

        * code: 401 Unauthorized -> Skill has not permission for this capability
 
Retrieve data from the server. The data is selected by the capabilities *Note:* Are those capabilities only implemented by the server or can the clients receive/send any info?


## Shutdown

*POST* **Server/vap/skillRegistry/skillClose** (Confirmable: Mandatory, Skill -> Registry)
 * skillId: (The one like org.company.product)

**Answer**:
* Either:
    * OK! (Code: 202 Deleted)
    * Error:
        * 400 Bad request -> skillId does not exist
            * code: int -> 400
            * type: "wrong skillId"
            * object: String -> The skillId that didn't exist
        * 401 Unauthorized: This skillId is not related to this address #!TODO#! Should this be emptier to make DDOS attacks more difficult?
            * code: int -> 401
            * type: "unauthorized"

# Discovery

Skills will search for the skill registry using CoAP's service discovery if no
IP is known.

Name of the CoAP service for the Skill Registry "vap-skill-registry"

# Connection

## Init

*POST* **Server/vap/skillRegistry/connect** (Confirmable: Mandatory, Skill -> Registry)
* name: Human readable name of the skill
* id: Unique ascii based name of the skill in the form of org.company.product
* vapVersion:
* uniqueAuthenticationToken (Optional)

Skill connects with skill register.

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
*POST* **Server/vap/skillRegistry/registerIntents** (Confirmable: Mandatory, Skill -> Registry)
* skillId: String
* nluData:[] -> One set per language
    * language: Language
    * intents: [] ->
        * name: String
        * utterances: [] ->
            * text: String
        * slots:  [] ->
            * name: String
            * entity: String -> Name of entity.
            * required: bool -> If true the assistant must ask for it beforehand.
            * prompt: Optional\<String> -> If required and not present the slot will be asked with this.
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

*GET (Obeserve)* **Server/vap/skillRegistry/skills/{skill_system_id}**  (Confirmable: Optional, needs to be observed)

*Empty payload*

**Answer:**
Initial Get request
    * skillId: String
    * request:
        * type: "event"|"intent"|"canYouAnswer"|"standalone" -> Internally tagged request data.
        * capabilities: [] ->
            * name: String
            * \<capability data>

        * type: "event"
        * event: String

        * type: "intent"
        * intent: String
        * locale: Language
        * input: String
        * slots: [] ->
            * name: String
            * value: Optional\<String>

Requests are answered via notifications. This is their structure:

**If type == "canYouAnswer":**
The server is asking "Can you answer this request?" When multiple skills are capable of answering the same request (it could be a generic request like "turn off the kitchen lights") we send them the intent and the slots (like "turn_off" "kitchen lights") and each skill returns an estimation of how well they can answer this request.

    (Code: 205 Content)
    * request_id: u64
    * confidence: float

**If type == "intent" or type == "event":**:
    (Code: 204 Changed, a skill can return "205 Content" if no internal state changed and the same request will always return the same answer).
    * request_id: u64
    * capabilities:
        * name: String
        * <capability data>

*DELETE* **Server/vap/skillRegistry/{skill_system_id}** (Confirmable: Mandatory, Skill -> Registry)

**Answer**:
* Either:
    * OK! (Code: 202 Deleted)
    * Error:
        * 400 Bad >request -> skillId does not exist
            * code: int -> 400
            * type: "wrong skillId"
            * object: String -> The skillId that didn't exist
        * 401 Unauthorized: This skillId is not related to this address.
            * code: int -> 401
            * type: "unauthorized"



## At any time:

*POST* **Server/vap/skillRegistry/notification** (Confirmable: Optional, Skill -> Registry)
* skillId: String
* data: (Can send to multiple at the same time, one per client to send)
    * type: String
    
    * type == "requested" -> As response to an intent or an event
    * requestId: String
    * capabilities:
        * name: String
        * <capability data>

    * type == "standalone" -> Sent on it's own volition by the skill
    * clientId: String -> SystemId of client
    * capabilities:
        * name: String
        * <capability data>

    * type == "canYouAnswer" -> Sent as a response to a canYouAnswer
    * confidence: float

 A notification is a message started by a skill. Sometimes a skill needs to send
 information on it's own, not as an answer to something the user asked, whether
 it's a stream of sound or whether is just an alarm. Note that the server can
 also receive notifications.

 **Answer:**
 * If the base message is ok: Code 204 Changed (Or 205 If everything inside is cacheable).

 * data:
    * type: String
    * code: int
    
    * type == "requested"  | type == "canYouAnswer"
        * requestId: String
    * type == "standalone"
        * clientId: String

* Errors:
    if type == "requested" | type == "canYouAnswer":
        * 402 Bad Option if the request does not exist or is not assigned to this skill

*GET* **Server/vap/skillRegistry/query** (Confirmable: Optional, Skill -> Registry)
* skillId : String
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


# TODOS

* Internally tagged vs externally tagged? (both Alexa and Hermes use internally tagged, what about for languages?)
* Better name than VAP?
* Signals and skill queries on VAP?
    * Task polling to detect changes? (Maybe this can be done as a separate skill)
* Context data in request (and maybe in "can answer"), user input, confidence, slots ...
* Should capabilities be written as organization.name (like vap.dyn_nlu or lily.custom_capability)?
* How to resolve a skill that changes server? Maybe we can use the zeroconf name? What alternative do we have if zeroconf is optional?
* The capabilities of the client are needed for the CanAnswer?
* Think on addresses of registries, if making them not to change would be an improvement on security, if so by how much?.
* TCP or UDP?
* Given that any path does not implement two methods, how about they are ignored?
* Should we join server queries and notifications using GET/POST into just one path? Queries would use GET and notifications POST.
* Instead of confirmable, can we just wait for the answer?
* How to treat versions? As Strings, as some combination of numbers...
* What is the type of the UniqueAuthentcationToken
* snake_case or CamelCase? (snake_case is the default for HTTP, Hermes and Alexa use pascalCase)
* pascalCase for URIs?
* Maybe the UniqueAuthenticationToken can be adapted/related to keys in OSCORE/DTLS?

## To resolve by myself

* Specify language negotiation strategy (first analyze them and then show possible answers). Available in fluent-langneg:
    * Exact match
    * Locale range
    * Use ICU LikelySubtags
    * Different variant of the same locale
    * 

## To check by others

"vap-skill-registry" service name

*Note:* The path shown on each message is the CoAP URL, Server means is a path the server is listening on while Skill is the same for any skill.

# Location:
Skills will search for the skill registry using zeroconf a.k.a bonjour a.k.a mDNS/DNS-SD

Name of the Zeroconf service for the Skill Registry "vap-skill-registry.udp"


# Connection

## Init:

*POST* **Server/vap/skill_registry/connect** (Confirmable: Mandatory, Skill -> Registry)
* name: Human readable name of the skill
* id: Unique ascii based name of the skill in the form of org.company.product
* vapVersion:
* uniqueAuthenticationToken (Optional)

Skill connects with skill register.

**Answer**
* One of:
    * OK! (Code: 201 Created)
        * langs: array[languages] -> Which languages are present in the system
        * uniqueAuthenticationToken (If not provided before)
    * Error  UniqueAuthenticationToken incorrect (either because of mismatch or because we sent an empty field but one was registered)


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
*POST* **Server/vap/skill_registry/register_utts** (Confirmable: Mandatory, Skill -> Registry)
* nluData: One set per language
    * utterances:
        * text: String
        * slots:
    * entities:

**Answer**
* Either:
    * Ok (Code: 201 Created)
    * Error
        * Why? 

## Skill interactions:
*GET* **Skill/vap/can_you_answer**  (Confirmable: Optional, Registry -> Skill)
* One of:
    * Event
        * Name
    * Request
        * Utterance
        * Slots

The server is asking "Can you answer this request?"

**Answer:** (Code: 205 Content)
* Confidence: float

 When multiple skills are capable of answering the same request (it could be a
 generic request like "turn off the kitchen lights") we send them the intent and the slots (like "turn_off" "kitchen lights") and each skill returns an 
 estimation of how well they can answer this request.

*POST* **Skill/vap/request** (Confirmable: Optional, Registry->Skill)
* Device:
    * Capabilities
        * Name (*TODO:* Internally or externally tagged?)
        * Version
    * Client name? (*NOTE:* So that the skill can record it if needed to send notifications later? )
* One of:
    * Event:
        * Name
    * Request:
        * Utterance
        * Slots
        * Language: Language in which the interaction happened

**Answer:** (Code: **TODO**)
    * Data:
        * One per capability:
            * Name: (*NOTE:* Here having the capability name on the outside looks easier for the server to filter even unknown capabilities, still to be confirmed)
                * Capability data: Any

## At any time:
*POST* **Server/vap/skill_registry/notification** (Confirmable: Optional, Skill -> Registry)
* Data: (Can send to multiple at the same time, one per client to send)
    * Client name to send this to
    * One per capability:
        * Name: (*Note:* same as above)
            * Capability data: Any
    

 A notification is a message started by a skill. Sometimes a skill needs to send
 information on it's own, not as an answer to something the user asked, whether
 it's a stream of sound or whether is just an alarm. Note that the server can
 also receive notifications.

**TODO:**  Does this needs an answer?


*GET* **Server/vap/skill_registry/query** (Confirmable: Optional, Skill -> Registry)
 * Query:
    * Data:
        * 
 
**Answer:**
 * Ok! (Code: 205 Content)
 * Error
    * **TODO:** Which errors?

Retrieve data from the server. *NOTE* Which data?

## Shutdown
*POST* **Server/vap/skill_registry/skill_close** (Confirmable: Mandatory, Skill -> Registry)
 * skill_id: (The one like org.company.product)

* Answer?
    * Whether it worked?

# Capabilities

## Log

This capability would let you log things into the server. They can be seen by the server and by the user. How they are presented to the user is up to to both the server and the clients.

Expected to be received as notifications.

## Dynamic NLU

Add Utterances after the initial set is sent, this is specially interesting for services like music streaming or home automation that the names aren't know until runtime, or even that the can change.

Expected to be received as notifications.
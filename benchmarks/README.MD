# Protocols Benchmark

The foundation of any voice assistant is the architecture and the connection
between the parts it's made of. If the assistant is wonderful but the connection
in between pieces is bad, overcomplex or even outdated, the experience itself
will be degraded, as hiccups, slowdowns, or things are impossible to do may arise.

## Vocabulary

**TLS**: *Transport Layer Security*. Wraps another protocol giving it 
authentacion properties. Works only on *TCP*.

**DTLS**: *Datagram Transport Laye Security*. Same as TLS but on *UDP*.
*TCP*-based protocols use *TLS* and *UDP*-based protocols use *DTLS* (if they
are compatible). *Note:* It seems that it can be problematic to use over the 
internet: [https://iot.stackexchange.com/questions/3028/solutions-to-secure-mqtt-sn-over-udp-with-dtls-or-alternatives].

## Categories

* **CPU Overhead**: The CPU that a implementation of the protocol requires. *Note:
  This is not very important but is nice to keep track of it, also each 
  implementation will have different characteristics here*.

* **Network overhead**: How taxing is the protocol on the network, that is, how much
  bandwith it needs.

* **Resistance to bad networks**: How well the procotol performs under conditions of
  unreliable connections. *Like Wi-Fi networks with poor signal*.

* **Isolation**: *Related to security*. Does this protocol limit which components 
  can talk to what? E.G: A skill downloaded from the internet, supplanting the 
  user while contacting to another skill like a banking or purchases skill?

* **Encryption**: *Related to security*. Does this protocol include encryption? 
  Having it would stop eavesdroppers. Also, if it is available, it encrypts the
  whole message (including some of the metadata) or just the payload?

* **Mobile**: How well the protocol adapts to a constrained OS like a mobile one. 
  One key point to be taken into account is that the OS can kill a service 
  whenever.

* **NLU Definition**: Whether the protocol is designed to store/transmit NLU 
  definition data,that would be utterances, slots in a raw form (not compiled, 
  not in any trained model).

* **Binary capable**: Whether the protocol can send binary data, very useful for
  things like transmitting voice from clients or sounds from skills to clients. 
  This is *determinied by messaging format*, if it is incapable of transmitting
  binary data, then an alternative way of sending these datas must be found.

## Protocols

A lot of those aren't real protocols but mockup protocols made with an
underlying application protocol (MQTT, CoAP ...) and a messaging format (JSON,
Flexbuf ...) this is what will determine a big chunk of their characteristics.

|        Protocol       | Based on | CPU Overhead | Network Overhead | Resistance to bad networks | Isolation | Full-Encryption | Mobile | NLU Definition | Binary capable |
|-----------------------|----------|--------------|------------------|----------------------------|-----------|-----------------|--------|----------------|----------------|
|  MQTT + Flexbuffers   |    TCP   |              |                  |                            |      ☐    |         🗹       |        |       🗹1      |        🗹       |
|    MQTT + MapBuf      |    TCP   |              |                  |                            |      ☐    |         🗹       |        |       🗹1      |        🗹       |
|    MQTT + MsgPack     |    TCP   |              |                  |                            |      ☐    |         🗹       |        |       🗹1      |        🗹       |
| MQTT-SN + FlexBuffers |    UDP   |              |                  |                            |      ☐    |         🗹†      |        |       🗹1      |        🗹       |
|   MQTT-SN + MapBuf    |    UDP   |              |                  |                            |      ☐    |         🗹†      |        |       🗹1      |        🗹       |
|   MQTT-SN + MsgPack   |    UDP   |              |                  |                            |      ☐    |         🗹†      |        |       🗹1      |        🗹       |
|  CoAp + FlexBufffers  |  UDP/TCP |              |                  |                            |      🗹    |        🗹*      |        |       🗹1      |        🗹       |
|     CoAp + MapBuf     |  UDP/TCP |              |                  |                            |      🗹    |        🗹*      |        |       🗹1      |        🗹       |
|     CoAp + MsgPack    |  UDP/TCP |              |                  |                            |      🗹    |        🗹*      |        |       🗹1      |        🗹       |
| Hermes (MQTT + JSON)  |    TCP   |              |                  |                            |      ☐    |         🗹       |        |       ☐2      |       🗹3       |
|   HiveMind/Mycroft    |    TCP   |              |                  |                            |      ☐    |         ☐       |        |        ?       |        ?       |

*Full-Encryption:*

* 🗹 Supports full-encryption through TLS
* ☐ Supports payload encryption (Protocol metadata like topic, can still be seen)


*Remarks*

**\*** CoAP supports TLS over TCP and DTLS over UDP
**†** DTLS
**1** Since this is a new protocol, it will be part of it, of course, though 
      the specifics are yet to be described.
**2** Original Hermes, has no support for NLU data in it, how it worked was by
      having that data precompiled and being part of the assistant itself.
**3** JSON is a text-based messaging format, which means 

### TCP vs UDP

*TCP* creates a connection, this means more overhead, yes, but it also means 
that intermediary routers are aware of this connection and can resend packets if
needed. There's also bandwith control, but the efficacy of this seems *arguable, 
there are cases where it seems ineffective* **TODO: Read paper about this**.

### Messaging protocols

The messaging protocols, the underlying protocol that actually carry over the
whole message, is what determines how it behaves on bad networks, if the skills 
and clients can be *isolated*, if it can be fully-encrypted, whether you need to
use TCP or UDP and it also has an effect on CPU overhead.

As it can be seen it is an important factor in how well the whole protocol
responds.

There are a ton of protocols there that we can use but we are only going to
consider three: CoAP, MQTT, and it's UDP version MQTT-SN. While there are others
like DDS or HTTP, we are only going to consider those which have a small 
footprint and target IoT. DDS is another option that we could take into account,
however it is a complex protocol ,including access to content, and doesn't work
well at all for constrained environments, for example there are no
implementations for an Arduino.

Some details about each protocol:

* **MQTT**: The defacto IoT standard. It is a pub-sub protocol which is 
  centralized (needs a program that distributes the messages, called the 
  broker).Being only pub-sub means that isolation of skills/clients is not
  possible, and being centralized means that every step means having to jump one
  more time (if we send from A to B, the path would be A->Broker->B each time).
  It is also only available for *TCP*.Two interesting features of MQTT are
  *retain*, in which a message is flagged to be *retained* inside the broker and
  available for later access and *last will* a message that will be
  automatically sent when a client loses it's connection.

* **MQTT-SN**: A version of MQTT for even more constrained environments. It 
  relies on *UDP* instead of *TCP*. This makes it lighter and possibly less
  taxing on the network, as an extra it produces smaller network packets as
  instead of topic names consisting of the whole identifier (i.e: 
  "/voice_protocol/v1/asr/asr_answer") a numeric id is used.

* **CoAP**: A promising "new" IoT protocol. It implements the same operations 
  HTTP do, and thuys is perfect for response/answer scenarios. It also adds the
  possibility of pub-sub,like MQTT, but it makes it better by not relying on 
  brokers, making it possible to be used in pretty much any scenario. Unlike
  other protocols it is available for both *TCP* and *UDP*. It's cons are that
  since it is a younger protocol it is less known and also, from what I've read
  the pub-sub part does not have strong mechanisms to make sure that messages
  arribe at the other end.

* **Hermes**: The current voice assistant protocol. It was made by Snips for
  it's voice asistant. It relies on MQTT + JSON and it was conceived as a protocol
  for a voice assistant where the core is managed as a set of microservices,
  Hermes being the connection between them.

* **HiveMind/Mycroft**: 

## Some extras

One of the planned new features for the new protocol is the creation of multiple
ad-hoc sub-protocols called which for the lack of a better term is called *capabilities*.

The idea is for the clients to announce what they can/cannot do, making them modular.
Enabling the possibility of having a text-only interface, having GUI interaction, or
even OS integration from remote (one could think of things like asking for a file
to be created or modified).

The only thing that comes close is the extra field in some of *Hermes* messages.
However, it is a string and would need a second parsing.


# Links

You can compare the message formats results here to what others have found in the
realm of Rust:

[https://blog.logrocket.com/rust-serialization-whats-ready-for-production-today/]
[https://github.com/llogiq/serdebench]

# Bibliography

TCP mechanisms to congestion and network failures
[https://www.ccexpert.us/traffic-shaping-2/tcp-and-udp-reactions-to-packet-loss.html]

DDS (and other protocols) definitions and meta-reasearch
[https://arxiv.org/pdf/1804.01747.pdf]

Comparison of MQTT and CoAP
[https://www.researchgate.net/publication/332853332_NDN_CoAP_and_MQTT_a_comparative_measurement_study_in_the_IoT]
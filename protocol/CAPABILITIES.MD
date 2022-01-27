# Capabilities

Capabilities are a central concept in VAP, allowing it to be modular, and adapt to new requirements. Capabilities are actually versioned, a skill or client could reject a version of a capability too old or too new.

## Some capabilities

### Sound

Englobes the ability of a device to record and reproduce sound. This accounts for both, voice (like incoming from the TTS) as well as sound data. Sound will be sent as OPUS, the best codec for voice. Is this better as OGG OPUS or just OPUS?


### Text

Ability to send and receive text.


### Image

Ability to receive (clients) and send (skills) pictures.


### Wakeword sync

The client will receive it's wakeword model from the server.


### Wakeword audio

The client uses wakeword as it's activation system, this skill will send the trimmed audio recorded from the wakeword so that analysis like a second pass wakeword or even voice fingerprint-based authentication.


### Log

This capability would lets send logs into the server. They can be seen by the server and by the user. How they are presented to the user is up to to both the server and the clients.

Expected to be received as notifications.

### Dynamic NLU

Add Utterances after the initial set is sent, this is specially interesting for services like music streaming or home automation that the names aren't know until runtime, or even that the can change.

Expected to be received as notifications.

## Some other ideas

* **Answer**: An skill-only capability. Serves to answer a query with text, if sound is supported a TTS-answer will be made, if text is supported a text-based answer will be sent (both if both are supported). This might not need to be a thing, does this one make sense?
* **Dynamic NLU**: Some data it's not known until runtime (e.g: your light bulbs), send it to the NLU for it to be recognized by the NLU, another alternative would be to be able to plug the skill as part of the resolution mechanism.

* **Cameras**: Computer Vision-related, how to do this?
* **GUI**: Some GUI, we most possibly need a markup, a custom one? HTML?

## Versioning mechanism

**TODO!**:
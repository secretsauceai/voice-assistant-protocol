# Capabilities

Capabilities are a central concept in VAP, allowing it to be modular, and adapt to new requirements. Capabilities are actually versioned, a skill or client could reject a version of a capability too old or too new.

## Some capabilities

* **Sound**: Englobes the ability of a device to record and reproduce sound. This accounts for both, voice (like incoming from the TTS) as well as sound data. Sound will be sent as OPUS, the best codec for voice. Is this better as OGG OPUS or just OPUS?
* **Text**: Ability to send and receive text.
* **Image**: Ability to receive (clients) and send (skills) pictures.
* **Wakeword sync**: The client will receive it's wakeword model from the server.
* **Wakeword audio**: The client uses wakeword as it's activation system, this skill will send the trimmed audio recorded from the wakeword so that analysis like a second pass wakeword or even voice fingerprint-based authentication.
* **Log**:
* **Dynamic NLU**:

## Some other ideas

* **Answer**: An skill-only capability. Serves to answer a query with text, if sound is supported a TTS-answer will be made, if text is supported a text-based answer will be sent (both if both are supported). This might not need to be a thing, does this one make sense?

* **Cameras**: Computer Vision-related, how to do this?
* **GUI**: Some GUI, we most possibly need a markup, a custom one? HTML?

## Versioning mechanism

**TODO!**:
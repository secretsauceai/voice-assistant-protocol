# VAP: Voice Assistant Protocol

A protocol for communicating with voice assistant clients and voice assistant skills.

Here you will find:

* **Benchmarks:** Inside the `benchmarks`folder, comparing the technology behind the protocol and how it relates to other technologies and established protocols.

* **Specification:** Inside the `protocol` folder, the specification itself and how it works.

* **Implementations:**Inside `vap-client-register`, `vap-common`, `vap-common-client`, `vap-common-skill`, `vap-skill-register` and `vap-python-skill` folders are the reference implementations. The skill client code is in Python, for better accessibility while everything else is in Rust for running it even on devices with limited capabilities.

## Implementations


### Important
* `vap-client-register`: An example register for clients.
* `vap-skill-register`: An example register for skills.
* `vap-python-skill`: An example Python skill. Python.

### Auxiliary
* `vap-common-client`: Structures and functions common for the clients themselves and the client registry.
* `vap-common`: Structures and functions common to both clients and skills.
* `vap-common-skill`: Structures and functions common for the skill themselves and the skill registry.
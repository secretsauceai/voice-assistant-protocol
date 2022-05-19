# VAP: Voice Assistant Protocol

A protocol for communicating with voice assistant clients and voice assistant skills.

Here you will find:

* **Benchmarks:** Inside the `benchmarks`folder, comparing the technology behind the protocol and how it relates to other technologies and established protocols.

* **Specification:** Inside the `protocol` folder, the specification itself and how it works.

* **Implementations:** Inside `vap-client-register`, `vap-common`, `vap-common-client`, `vap-common-skill`, `vap-skill-register`, `vap-skill-framework` and `vap-python-skill` folders are the reference implementations. The skill client code is in Python as well as Rust, for better accessibility while everything else is in Rust for running it even on devices with limited capabilities. Note: The skill client code in Python is meant as a simple example implementing directly VAP, do note that the implementation there does not cover every single error handling of the protocol.

## Implementations


### Important
* `vap-client-register-demo`: An example register for clients.
* `vap-skill-register-demo`: An example register for skills.
* `vap-skill-framework-demo`: An example VAP skill using `vap-skill-framework`.
* `vap-python-skill`: An example Python skill. Python.

### Auxiliary
* `vap-common-client`: Structures and functions common for the clients themselves and the client registry.
* `vap-common`: Structures and functions common to both clients and skills.
* `vap-common-skill`: Structures and functions common for the skill themselves and the skill registry.
* `vap-client-register`: The Rust-based client register itself.
* `vap-skill-register`: The Rust-based skill register itself.
* `vap-skill-framework`: A library for writing Rust-based skills.
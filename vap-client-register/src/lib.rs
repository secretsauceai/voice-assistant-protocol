#![no_std]
#![allow(dead_code)]
extern crate alloc;
use alloc::{format, string::String, vec::Vec};
use coap_lite::{CoapRequest, CoapResponse, MessageClass, MessageType, Packet, RequestType};

use core;
use embedded_nal::{TcpClientStack, UdpClientStack};
use no_std_net::ToSocketAddrs;
use vap_common::capability::CapabilityCode;

pub struct VAPClient<Endpoint> {
    endpoint: Endpoint,
    name: String,
    id: String,
    vap_version: String,
}
/// Structure of Request:
/// *POST* **Server/vap/clientRegistry/connect** (Confirmable: Mandatory, Client -> Registry)
///* name:
///* id: String -> (like org.company.product)
///* vapVersion:
///The server will answer a UniqueAuthenticationToken
/// only if this is the first time the client is
/// connecting and we don't have any record of it.
struct ConnectRequest<Endpoint>(CoapRequest<Endpoint>);

impl<Endpoint> ConnectRequest<Endpoint> {
    pub fn new(endpoint: Endpoint, name: String, id: String, vap_version: String) -> Self {
        let mut packet = Packet::new();
        packet.header.set_type(MessageType::Confirmable);
        packet.header.code = MessageClass::Request(RequestType::Post);

        packet.payload = format!("Capabilities:").into();
        let mut req = CoapRequest::from_packet(packet, endpoint);
        req.set_path("Server/vap/clientRegistry/connect");
        Self(req)
    }
}
// **Answer**
// * One of:
//     * OK! (Code: 201 Created)
//         * locales: Locale
//     * Errors:
//         * 400 Bad Request: Id already exists? (should Ids be unique?)
//         * 400 Bad Request: vapVersion incompatible
//         * 401 Unauthorized: connection denied by policy or by the user (maybe the user didn't accept the client or it is blocked)
//             * code = 401
//             * type = "connectionDenied"

///Structure of Request:
/// *POST* **Server/vap/clientRegistry/sessionStart** (Confirmable: Optional, Client -> Registry)
///* capabilities: Optional<[]> ->
///    * name: String
///    * <capability data>
///* exactTimeStamp:?
///This signals that a client wants to start a session. At this point we can send capabilities
/// too, they are meant for user authorization and wake word double checking (with a bigger,
/// slower, more accurate model in the server). Of course, the server is free to either accept
/// it or reject if because of any reason.
struct SessionStartRequest<Endpoint>(CoapRequest<Endpoint>);

impl<Endpoint> SessionStartRequest<Endpoint> {
    pub fn new(endpoint: Endpoint, capabilities: Option<&[CapabilityCode]>) -> Self {
        let mut packet = Packet::new();
        packet.header.set_type(MessageType::Confirmable);
        packet.header.code = MessageClass::Request(RequestType::Post);
        if let Some(capabilities) = capabilities {
            packet.payload = Vec::from_iter(capabilities.iter().map(|c| c.to_u8())).into();
        }
        let mut req = CoapRequest::from_packet(packet, endpoint);
        req.set_path("Server/vap/clientRegistry/sessionStart");
        Self(req)
    }
}

/// Structure of Request:
/// *POST* **Server/vap/clientRegistry/sessionData** (Confirmable: Optional, Client -> Registry)
/// * capabilities: [] ->
///     * name: String
///     * <capability data>
/// * lastFragment: bool

struct SessionDataRequest<Endpoint>(CoapRequest<Endpoint>);
//TODO: make this generic over the type of data, conversion to
// Vec<u8> should be handled within the constuction of the request
//if possible
impl<Endpoint> SessionDataRequest<Endpoint> {
    pub fn new(endpoint: Endpoint, data: Vec<u8>, end_session: bool) -> Self {
        let mut packet = Packet::new();
        packet.header.set_type(MessageType::Confirmable);
        packet.header.code = MessageClass::Request(RequestType::Post);
        packet.payload = data.into();
        let mut req = CoapRequest::from_packet(packet, endpoint);
        req.set_path("Server/vap/clientRegistry/sessionData");
        Self(req)
    }
}
struct clientCloseRequest<Endpoint>(CoapRequest<Endpoint>);

impl<Endpoint> clientCloseRequest<Endpoint> {
    pub fn new(endpoint: Endpoint, id: String) -> Self {
        let mut packet = Packet::new();
        packet.header.set_type(MessageType::Confirmable);
        packet.header.code = MessageClass::Request(RequestType::Post);
        packet.payload = id.into();
        let mut req = CoapRequest::from_packet(packet, endpoint);
        req.set_path("Server/vap/clientRegistry/clientClose");
        Self(req)
    }
}

// enum ConnectResponseClass {
//     Ok,
//     Error(ConnectError),
// }

// *POST* **Server/vap/clientRegistry/connect** (Confirmable: Mandatory, Client -> Registry)
// * name:
// * id: String -> (like org.company.product)
// * vapVersion:

// The server will answer a UniqueAuthenticationToken only if this is the first time the client is connecting and we don't have any record of it.

use coap_lite::{CoapRequest, CoapResponse, MessageClass, MessageType, Packet, RequestType};
use no_std_net::SocketAddr;
pub struct VAPClient<Endpoint> {
    endpoint: Endpoint,
    name: String,
    id: String,
    vap_version: String,
}

impl<Endpoint> VAPClient<Endpoint> {
    pub fn new(endpoint: Endpoint, name: String, id: String, vap_version: String) -> Self {
        Self {
            endpoint,
            name,
            id,
            vap_version,
        }
    }
    pub fn connect(&self, socket: SocketAddr) -> Result<Locale, Error> {
     
    let mut packet = Packet::new();
    packet.header.set_type(MessageType::Confirmable);
    packet.header.code = MessageClass::Request(RequestType::Post);

    packet.payload = format!("name={},id={},vapVersion={}", self.name, self.id, self.vap_version).into();
    let mut req = CoapRequest::from_packet(packet, self.endpoint);
    
    req.set_path("Server/vap/clientRegistry/connect");
    
    socket.send_to(req.message.to_bytes(), self.endpoint).unwrap();

    
    let mut res = 
    println!("Response: {:?}", res);
    let payload = res.get_payload().unwrap();
    println!("Payload: {:?}", payload);
    }

}

// *POST* **Server/vap/clientRegistry/connect** (Confirmable: Mandatory, Client -> Registry)
// * name:
// * id: String -> (like org.company.product)
// * vapVersion:

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

// The server will answer a UniqueAuthenticationToken only if this is the first time the client is connecting and we don't have any record of it.
pub fn connect(endpoint: Endpoint, name: String, id: String, vap_version: String) -> Result<Locale, Error> {
    let mut packet = Packet::new();
    packet.header.set_type(MessageType::Confirmable);
    packet.header.code = MessageClass::Request(RequestType::Post);

    packet.payload = format!("name={},id={},vapVersion={}", name, id, vap_version).into();
    let mut req = CoapRequest::from_packet(packet, None)
    from_packet(packet, )
    req.set_method(RequestType::Post);
    req.set_path("Server/vap/clientRegistry/connect");

    req.set_payload("vapVersion".as_bytes().to_vec());
    let mut res = req.send("coap://localhost:5683").unwrap();
    println!("Response: {:?}", res);
    let payload = res.get_payload().unwrap();
    println!("Payload: {:?}", payload);
    Ok(Locale {})
}

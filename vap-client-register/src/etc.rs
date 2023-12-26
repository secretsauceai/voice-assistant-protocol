// trait CoapTransport {
//     fn send(&self, message: &CoapMessage) -> Result<(), std::io::Error>;
//     fn receive(&self) -> Result<CoapMessage, std::io::Error>;
// }

// impl CoapTransport for UdpClientStack {
//     fn send(&self, message: &CoapMessage) -> Result<(), std::io::Error> {
//         todo!()
//     }

//     fn receive(&self) -> Result<CoapMessage, std::io::Error> {
//         todo!()
//     }
//     // Implement the methods for UDP

// }

// impl CoapTransport for TcpClientStack {
//     fn send(&self, message: &CoapMessage) -> Result<(), std::io::Error> {
//         todo!()
//     }

//     fn receive(&self) -> Result<CoapMessage, std::io::Error> {
//         todo!()
//     }
//     // Implement the methods for TCP
// }
// struct InitializedNetworkStack<ToSocketAddrs> {
//     bind_addr: ToSocketAddrs,
//     peer_addr: ToSocketAddrs,
// }

// impl<ToSocketAddrs> UdpClientStack for InitializedNetworkStack<ToSocketAddrs> {
//     fn new(bind_addr: ToSocketAddrs, peer_addr: ToSocketAddrs) -> Self {
//         Self {
//             bind_addr,
//             peer_addr,
//         }
//     }

//     type UdpSocket;

//     type Error;

//     fn socket(&mut self) -> Result<Self::UdpSocket, Self::Error> {
//         todo!()
//     }

//     fn connect(
// 		    &mut self,
// 		    socket: &mut Self::UdpSocket,
// 		    remote: no_std_net::SocketAddr,
// 	    ) -> Result<(), Self::Error> {
//         todo!()
//     }

//     fn send(&mut self, socket: &mut Self::UdpSocket, buffer: &[u8]) -> embedded_nal::nb::Result<(), Self::Error> {
//         todo!()
//     }

//     fn receive(
// 		    &mut self,
// 		    socket: &mut Self::UdpSocket,
// 		    buffer: &mut [u8],
// 	    ) -> embedded_nal::nb::Result<(usize, no_std_net::SocketAddr), Self::Error> {
//         todo!()
//     }

//     fn close(&mut self, socket: Self::UdpSocket) -> Result<(), Self::Error> {
//         todo!()
//     }
// }

// fn init_udp(bind_addr: ToSocketAddrs, peer_addr: ToSocketAddrs) -> InitializedNetworkStack {
//     let socket = UdpClientStack::new(bind_addr, peer_addr);
//     InitializedTransport::Udp(socket)
// }

// impl<Endpoint> VAPClient<Endpoint> {
//     pub fn new(endpoint: Endpoint, name: String, id: String, vap_version: String) -> Self {
//         Self {
//             endpoint,
//             name,
//             id,
//             vap_version,
//         }
//     }
//     pub fn connect(&self, socket: UdpClientStack) -> Result<Locale, Error> {

//     let mut packet = Packet::new();
//     packet.header.set_type(MessageType::Confirmable);
//     packet.header.code = MessageClass::Request(RequestType::Post);

//     packet.payload = format!("name={},id={},vapVersion={}", self.name, self.id, self.vap_version).into();
//     let mut req = CoapRequest::from_packet(packet, self.endpoint);

//     req.set_path("Server/vap/clientRegistry/connect");

//     socke
//     send_to(req.message.to_bytes(), self.endpoint).unwrap();

//     let mut res =
//     println!("Response: {:?}", res);
//     let payload = res.get_payload().unwrap();
//     println!("Payload: {:?}", payload);
//     }

// }

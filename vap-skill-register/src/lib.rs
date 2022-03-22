use std::future::Future;
use std::io::Cursor;
use std::net::SocketAddr;

use coap_lite::{RequestType as Method, CoapRequest, CoapResponse};
use coap::{CoAPClient, Server};
use futures::{channel::{mpsc, oneshot}, StreamExt, SinkExt};
use libmdns::{Responder, Service};
use rmp_serde::from_read;
use serde::de::DeserializeOwned;
use thiserror::Error;
use vap_common_skill::structures::*;

pub use coap_lite::ResponseType;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    NameInvalid(std::io::Error),

    #[error("{0}")]
    ZeroconfServiceRegistration(std::io::Error),

    #[error("{0}")]
    ZeroconfPolling(std::io::Error),

    #[error("A Oneshot channel was closed")]
    ClosedChannel
}

pub struct Response {
    pub status: ResponseType,
    pub payload: Vec<u8>,
}

pub struct SkillRegister {
    name: String,
    port: u16,
    in_send: mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>,
}

pub enum SkillRegisterMessage {
    Connect(MsgConnect),
    RegisterIntents(MsgRegisterIntents),
    Notification(MsgNotification),
    Query(MsgQuery),
    Close(MsgSkillClose),
}

impl SkillRegister {
    pub fn new(name: &str, port: u16) -> Result<(Self, SkillRegisterStream), Error> {   
        let (in_send, in_recv) = mpsc::channel(20);
        Ok((
            SkillRegister {
                name: name.to_string(),
                port,
                in_send,
            },

            SkillRegisterStream {
                stream_in: in_recv,
            }
        ))
    }

    pub async fn run(&self) -> Result<(), Error>  {
        let _zeroconf = ZeroconfService::new(&self.name, self.port)?;

        async fn perform(request: CoapRequest<SocketAddr>, mut in_send: mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>) -> Option<CoapResponse> {
            fn read_payload<T: DeserializeOwned>(payload: &[u8], r: Option<CoapResponse>) -> Result<(T, Option<CoapResponse>), Option<CoapResponse>> {
                match from_read(Cursor::new(payload)) {
                    Ok::<T,_>(a) => {
                        Ok((a,r))
                    }
                    Err(e) => {
                        Err(r.map(|mut r|{
                            let status = match e {
                                rmp_serde::decode::Error::TypeMismatch(_) => {
                                    coap_lite::ResponseType::RequestEntityIncomplete
                                }

                                _ => {
                                    coap_lite::ResponseType::BadRequest
                                }
                            };

                            r.set_status(status);
                            r
                        }))
                    }
                }
            }

            fn response_not_found(r: Option<CoapResponse>) -> Option<CoapResponse> {
                r.map(|mut r| {
                    r.set_status(coap_lite::ResponseType::MethodNotAllowed);
                    r
                })
            }

            async fn handle_msg<T: DeserializeOwned, F: FnOnce(T) -> SkillRegisterMessage>(request: CoapRequest<SocketAddr>, in_send: &mut mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>, cb: F) -> Option<CoapResponse> {
                match read_payload(&request.message.payload, request.response) {
                    Ok::<(T,_),_>((p, resp)) => {
                        let (sender, receiver) = oneshot::channel();
                        in_send.send((cb(p), sender)).await.unwrap();
                        match receiver.await {
                            Ok(resp_data) => {
                                resp.map(|mut r|{
                                    r.set_status(resp_data.status);
                                    r.message = coap_lite::Packet::from_bytes(&resp_data.payload).unwrap();
                                    r
                                })
                            }
                            Err(_) => {
                                None
                            }
                        }
                    }
                    Err(r) => {
                        r
                    }
                }
            }

            
            match *request.get_method() {
                Method::Get => {
                    match request.get_path().as_str() {
                        "vap/skillRegistry/query" => {
                            handle_msg(request, &mut in_send, |p|{SkillRegisterMessage::Query(p)}).await
                        }

                        _ => response_not_found(request.response)
                    }
                }

                Method::Post => {                 
                    match request.get_path().as_str() {
                        "vap/skillRegistry/connect" => {
                            handle_msg(request, &mut in_send, |p|{SkillRegisterMessage::Connect(p)}).await
                        }

                        "vap/skillRegistry/registerIntents" => {
                            handle_msg(request, &mut in_send, |p|{SkillRegisterMessage::RegisterIntents(p)}).await
                        }

                        "vap/skillRegistry/notification" => {
                            handle_msg(request, &mut in_send, |p|{SkillRegisterMessage::Notification(p)}).await
                        }

                        "vap/skillRegistry/skillClose" => {
                            handle_msg(request, &mut in_send, |p|{SkillRegisterMessage::Close(p)}).await
                        }

                        _ => response_not_found(request.response)
                    }                    
                }

                _ => {
                    println!("request by other method");
                    request.response.map(|mut r|{
                        r.set_status(coap_lite::ResponseType::MethodNotAllowed);
                        r  
                    })
                },
            }
        }

        let mut server = Server::new(format!("127.0.0.1:{}", self.port)).unwrap();
        server.run( |request| {
            perform(request, self.in_send.clone())
        }).await.unwrap();
        Ok(())   
    }

    pub fn skills_answerable(&mut self, ips: &[String]) -> Vec<MsgSkillCanAnswerResponse> {
        fn send_msg(ip: &str) -> Result<MsgSkillCanAnswerResponse, Error> {
            let c = CoAPClient::new(ip).unwrap();
            let msg = MsgSkillCanAnswer{};
            let data = rmp_serde::to_vec(&msg).unwrap();
            let resp = c.request_path("vap/canYouAnswer", Method::Get, Some(data), None).unwrap();
            let resp_data = rmp_serde::from_read(Cursor::new(resp.message.payload)).unwrap();
            Ok(resp_data)
        }

        let mut answers = Vec::new();
        for ip in ips {
            match send_msg(ip) {
                Ok(resp) => {
                    println!("{:?}", resp);
                    answers.push(resp);
                }
                Err(e) => {
                    // TODO: What to do with the errors?
                    println!("{:?}", e);
                }
            }
        }
        
        answers
    }

    pub async fn activate_skill(&mut self, ip: String, msg: MsgSkillAnswer) -> Result<MsgSkillAnswerResponse, Error> {
        let c = CoAPClient::new(ip).unwrap();
        let data = rmp_serde::to_vec(&msg).unwrap();
        let resp = c.request_path("vap/canYouAnswer", Method::Get, Some(data), None).unwrap();
        let resp_data = rmp_serde::from_read(Cursor::new(resp.message.payload)).unwrap();
        Ok(resp_data)
    }
}

pub struct SkillRegisterStream {
    stream_in: mpsc::Receiver<(SkillRegisterMessage, oneshot::Sender<Response>)>,
}

impl SkillRegisterStream {
    pub async fn recv(&mut self) -> Result<(SkillRegisterMessage, oneshot::Sender<Response>), Error> {
        Ok(self.stream_in.next().await.unwrap())
    }

    pub async fn read_incoming<F, Fut>(mut self, cb: F) -> Result<(), Error>
    where
    F: Fn(SkillRegisterMessage) -> Fut,
    Fut: Future<Output = Response> {
        loop {
            let (msg, sender) = self.recv().await.unwrap();
            sender.send(cb(msg).await).map_err(|_|Error::ClosedChannel)?;
        }
    }
}

struct ZeroconfService {
    _responder: Responder,
    _service: Service,
}

impl ZeroconfService {
    fn new(name: &str, port:u16) -> Result<ZeroconfService, Error> {
        let _responder = Responder::new().map_err(Error::ZeroconfServiceRegistration)?;
        let _service = _responder.register(
            "_vap-skill-register._udp".to_owned(), 
            name.to_owned(),
            port,
            &["path=/"]
        );
    
        Ok(ZeroconfService{_responder,_service})
    }
}

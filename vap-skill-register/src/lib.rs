use std::cell::RefCell;
use std::{io::Cursor, collections::HashMap};
use std::net::SocketAddr;
use std::sync::Arc;

use coap_lite::{RequestType as Method, CoapRequest, CoapResponse};
use coap::{CoAPClient, Server};
use futures::{channel::{mpsc, oneshot}, StreamExt, SinkExt, lock::Mutex};
use rmp_serde::from_read;
use serde::de::DeserializeOwned;
use thiserror::Error;
use vap_common_skill::structures::*;
use vap_common_skill::structures::{msg_skill_request::{ClientData, RequestData}, msg_notification::Data};

pub use coap_lite::ResponseType;
pub use vap_common_skill::structures as structures;


type RequestId = u64;
type SharedPending<D> = Arc<
    Mutex<
        HashMap<RequestId, oneshot::Sender<
                D
            >
        >
    >
>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("A Oneshot channel was closed")]
    ClosedChannel
}

pub struct Response {
    pub status: ResponseType,
    pub payload: Vec<u8>,
}

pub struct SkillRegister {
    name: String,
    ip_address: String,
    in_send: mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>,
    pending_requests: SharedPending<Vec<PlainCapability>>,
    pending_can_you: SharedPending<f32>
}

pub enum SkillRegisterMessage {
    Connect(MsgConnect),
    RegisterIntents(MsgRegisterIntents),
    Notification(MsgNotification),
    Query(MsgQuery),
    Close(MsgSkillClose),
}

impl SkillRegister {
    pub fn new(name: &str, port: u16) -> Result<(Self, SkillRegisterStream, SkillRegisterOut), Error> {   
        let (in_send, in_recv) = mpsc::channel(20);
        let ip_address = format!("127.0.0.1:{}", port);
        let client = CoAPClient::new(&ip_address).unwrap();
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let pending_can_you = Arc::new(Mutex::new(HashMap::new()));
        Ok((
            SkillRegister {
                name: name.to_string(),
                ip_address: format!("127.0.0.1:{}", port),
                in_send,
                pending_requests: pending_requests.clone(),
                pending_can_you: pending_can_you.clone()
            },

            SkillRegisterStream {
                stream_in: in_recv,
            },

            SkillRegisterOut {client, pending_requests, 
                pending_can_you, next_request: RefCell::new(0)}
        ))
    }

    pub async fn run(self) -> Result<(), Error>  {
        async fn perform(
            request: CoapRequest<SocketAddr>,
            mut in_send: mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>,
            pending_requests: &SharedPending<Vec<PlainCapability>>,
            pending_can_you: &SharedPending<f32>
        ) -> Option<CoapResponse> {
            fn read_payload<T: DeserializeOwned>(payload: &[u8], r: Option<CoapResponse>) -> Result<(T, Option<CoapResponse>), Option<CoapResponse>> {
                match from_read(Cursor::new(payload)) {
                    Ok::<T,_>(a) => {
                        Ok((a,r))
                    }
                    Err(e) => {
                        Err(r.map(|mut r|{
                            println!("{}", &e);
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

            async fn wait_response(
                receiver: oneshot::Receiver<Response>,
                resp: Option<CoapResponse>
            ) -> Option<CoapResponse> {
                match receiver.await {
                    Ok(resp_data) => {
                        resp.map(|mut r|{
                            r.set_status(resp_data.status);
                            r.message.payload = resp_data.payload;
                            r
                        })
                    }
                    Err(_) => {
                        None
                    }
                }  
            }

            async fn handle_msg<T: DeserializeOwned, F>(
                request: CoapRequest<SocketAddr>,
                in_send: &mut mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>,
                cb: F,
            ) -> Option<CoapResponse> where
                F: FnOnce(T) -> SkillRegisterMessage{

                match read_payload(&request.message.payload, request.response) {
                    Ok::<(T,_),_>((p, resp)) => {
                        let (sender, receiver) = oneshot::channel();
                        in_send.send((cb(p), sender)).await.unwrap();
                        wait_response(receiver, resp).await
                    }
                    Err(r) => {
                        r
                    }
                }
            }

            
            match *request.get_method() {
                Method::Get => {
                    if request.get_path().starts_with("vap/skillRegistry/skills/") {
                        request.response.map(|mut r| {
                            r.set_status(coap_lite::ResponseType::Content);
                            r.message.payload = vec![];
                            r
                        })
                    }

                    else {
                        match request.get_path().as_str() {
                            "vap/skillRegistry/query" => {
                                handle_msg(
                                    request,
                                    &mut in_send,
                                    |p|{SkillRegisterMessage::Query(p)}
                                ).await
                            }

                            ".well-known/core" => {
                                request.response.map(|mut r|{
                                    r.set_status(coap_lite::ResponseType::Content);
                                    r.message.payload = b"</vap>;rt=\"vap-skill-registry\"".to_vec();
                                    r
                                })
                            }

                            _ => {
                                if request.get_path().starts_with("vap/request/") {
                                    // TODO: Make sure only the same skill is asking for it.
                                    request.response.map(|mut r|{
                                        r.set_status(coap_lite::ResponseType::Valid);
                                        r
                                    })
                                } else {
                                    response_not_found(request.response)
                                }
                            }
                        }
                    }
                }

                Method::Post => {                 
                    match request.get_path().as_str() {
                        "vap/skillRegistry/connect" => {
                            handle_msg(
                                request,
                                &mut in_send,
                                |p|{SkillRegisterMessage::Connect(p)}
                            ).await
                        }

                        "vap/skillRegistry/registerIntents" => {
                            handle_msg(
                                request,
                                &mut in_send,
                                |p|{SkillRegisterMessage::RegisterIntents(p)}
                            ).await
                        }

                        "vap/skillRegistry/notification" => {
                            
                            match read_payload(&request.message.payload, request.response) {
                                Ok::<(MsgNotification,_),_>((msg, resp)) => {
                                    let (sender, receiver) = oneshot::channel();

                                    // XXX: Think of how to sort this when we have multiple requests of each kind
                                    for a in &msg.data {
                                        match a {
                                            Data::CanYouAnswer{request_id, confidence} => {
                                                // XXX: Redirect to it's request
                                                match pending_can_you.lock().await.remove(&request_id) {
                                                    Some(sender) => {
                                                        sender.send(*confidence).unwrap();
                                                    }
                                                    None => {
                                                        // XXX: Send error
                                                    }
                                                }
                                            }
                                            Data::Requested{request_id, capabilities} => {
                                                // XXX: Redirect to it's request
                                                match pending_requests.lock().await.remove(&request_id) {
                                                    Some(sender) => {
                                                        sender.send(capabilities.clone()).unwrap();
                                                    }
                                                    None => {
                                                        // XXX: Send error
                                                    }
                                                }
                                            }
                                            Data::StandAlone{client_id, capabilities} => {
                                                // XXX: Redirect to the in channel
                                                in_send.send((SkillRegisterMessage::Notification(msg.clone()), sender)).await.unwrap();
                                            }
                                        }
                                    }

                                    wait_response(receiver, resp).await
                                }
                                Err(r) => {
                                    r
                                }
                            }
                        }

                        _ => response_not_found(request.response)
                    }                    
                }
                Method::Put => {
                    // Puts are needed so that an observe update is produced
                    request.response.map(|mut r|{
                        r.set_status(coap_lite::ResponseType::Valid);
                        r
                    })
                }

                Method::Delete => {
                    if request.get_path().starts_with("vap/skillRegistry/skills/") {
                        // TODO: Verify the name in the path is the same as the name in the request.
                        handle_msg(
                            request,
                            &mut in_send,
                            |p|{SkillRegisterMessage::Close(p)}
                        ).await
                    }
                    else {
                        response_not_found(request.response)
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

        let mut server = Server::new(&self.ip_address).unwrap();
        server.enable_all_coap(0);
        server.run( |request| {    
            perform(request, self.in_send.clone(), &self.pending_requests, &self.pending_can_you).await
        }).await.unwrap();
        Ok(())
    }
}

pub struct SkillRegisterOut {
    client: CoAPClient,
    pending_requests: SharedPending<Vec<PlainCapability>>,
    pending_can_you: SharedPending<f32>,
    next_request: RefCell<RequestId>
}

impl SkillRegisterOut {
    pub fn skills_answerable(&self, ids: &[String], request: RequestData, client: ClientData) -> Vec<MsgNotification> {
        // TODO: Respond to the notification
        fn send_msg(coap_self: &CoAPClient, id: &str, request: RequestData, request_id: RequestId, client: ClientData) -> Result<MsgNotification, Error> {
            let msg = MsgSkillRequest {client, request_id, request};
            let data = rmp_serde::to_vec(&msg).unwrap();
            let path = format!("vap/skillRegistry/skills/{}", id);
            let resp = coap_self.request_path(&path, Method::Get, Some(data), None).unwrap();
            let resp_data = rmp_serde::from_read(Cursor::new(resp.message.payload)).unwrap();
            Ok(resp_data)
        }

        let mut answers = Vec::new();
        for id in ids {
            match send_msg(&self.client, id, request.clone(), self.get_id(), client.clone()) {
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

    fn get_id(&self) -> RequestId {
        let mut ref_id = self.next_request.borrow_mut();
        let id = *ref_id;
        *ref_id += 1;

        id
    }

    pub async fn activate_skill(&self, name: String, mut msg: MsgSkillRequest) -> Result<Vec<PlainCapability>, Error> {
        // TODO: Respond to the notification
        let req_id = self.get_id();
        msg.request_id = req_id;
        let (sender,receiver) = oneshot::channel();
        let data = rmp_serde::to_vec(&msg).unwrap();

        self.pending_requests.lock().await.insert(req_id, sender);

        let resp = self.client.request_path(&format!("vap/skillRegistry/skills/{}", name), Method::Put, Some(data), None).unwrap();
        assert_eq!(resp.get_status(), &coap_lite::ResponseType::Content);

        let resp_data = receiver.await.unwrap();
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
}
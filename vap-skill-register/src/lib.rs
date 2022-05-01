use std::cell::RefCell;
use std::{io::Cursor, collections::HashMap};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex as SyncMutex};

use coap_lite::{RequestType as Method, CoapRequest, CoapResponse};
use coap::{CoAPClient, Server};
use futures::future::{join, join_all};
use futures::{channel::{mpsc, oneshot}, StreamExt, SinkExt, lock::Mutex};
use rmp_serde::{from_read, to_vec_named};
use serde::de::DeserializeOwned;
use thiserror::Error;
use vap_common_skill::structures::*;
use vap_common_skill::structures::{msg_skill_request::{ClientData, RequestData}};

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

pub const VAP_VERSION: &str = "Alpha";
pub const SYSTEM_SELF_ID: &str = "vap.SYSTEM";

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
    ip_address: String,
    in_send: mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>,
    pending_requests: SharedPending<(Vec<PlainCapability>, oneshot::Sender<RequestResponse>)>,
    pending_can_you: SharedPending<f32>,
    current_skills: Arc<SyncMutex<HashMap<String, ()>>>
}

pub struct Notification {
    pub skill_id: String,
    pub data: Vec<NotificationData>
}

#[derive(Debug, Clone)]
pub struct NotificationData {
    pub client_id: String,
    pub capabilities: Vec<structures::PlainCapability>
}

pub enum SkillRegisterMessage {
    Connect(MsgConnect),
    RegisterIntents(MsgRegisterIntents),
    Notification(Notification),
    Query(MsgQuery),
    Close(MsgSkillClose),
}

fn respond(resp: Option<CoapResponse>, st: ResponseType, pl: Vec<u8>) -> Option<CoapResponse> {
    resp.map(|mut c|{
        c.set_status(st);
        c.message.payload = pl;
        c
    })
}

impl SkillRegister {
    pub fn new(port: u16) -> Result<(Self, SkillRegisterStream, SkillRegisterOut), Error> {   
        let (in_send, in_recv) = mpsc::channel(20);
        let ip_address = format!("127.0.0.1:{}", port);
        let client = CoAPClient::new(&ip_address).unwrap();
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let pending_can_you = Arc::new(Mutex::new(HashMap::new()));
        Ok((
            SkillRegister {
                ip_address: format!("127.0.0.1:{}", port),
                in_send,
                pending_requests: pending_requests.clone(),
                pending_can_you: pending_can_you.clone(),
                current_skills: Arc::new(SyncMutex::new(HashMap::new()))
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
            pending_requests: &SharedPending<(Vec<PlainCapability>, oneshot::Sender<RequestResponse>)>,
            pending_can_you: &SharedPending<f32>,
            current_skills: Arc<SyncMutex<HashMap<String, ()>>>
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
                respond(r, ResponseType::MethodNotAllowed, vec![])
            }

            async fn wait_response<F>(
                receiver: oneshot::Receiver<Response>,
                resp: Option<CoapResponse>,
                cb: F
            ) -> Option<CoapResponse> where
            F: FnOnce(&Response)
             {
                match receiver.await {
                    Ok(resp_data) => {
                        cb(&resp_data);
                        respond(resp, resp_data.status, resp_data.payload)
                    }
                    Err(_) => {
                        None
                    }
                }  
            }

            async fn handle_msg<T: DeserializeOwned, F, F2>(
                request: CoapRequest<SocketAddr>,
                in_send: &mut mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>,
                key_check: F2,
                cb: F,
            ) -> Option<CoapResponse> where
                F: FnOnce(T) -> SkillRegisterMessage,
                F2: FnOnce(&T) -> bool{

                match read_payload(&request.message.payload, request.response) {
                    Ok::<(T,_),_>((p, resp)) => {
                        if  key_check(&p){
                            let (sender, receiver) = oneshot::channel();
                            in_send.send((cb(p), sender)).await.unwrap();
                            wait_response(receiver, resp, |_|{}).await
                        }
                        else {
                            respond(resp, ResponseType::BadRequest, vec![])
                        }
                    }
                    Err(r) => {
                        r
                    }
                }
            }

            
            match *request.get_method() {
                Method::Get => {
                    if request.get_path().starts_with("vap/skillRegistry/skills/") {
                        respond(request.response, ResponseType::Content, vec![])
                    }

                    else {
                        match request.get_path().as_str() {
                            "vap/skillRegistry/query" => {
                                handle_msg(
                                    request,
                                    &mut in_send,
                                    |p: &MsgQuery|current_skills.lock().unwrap().contains_key(&p.skill_id),
                                    |p|{SkillRegisterMessage::Query(p)}
                                ).await
                            }

                            ".well-known/core" => {
                                respond(request.response, ResponseType::Content, b"</vap>;rt=\"vap-skill-registry\"".to_vec())
                            }

                            _ => {
                                if request.get_path().starts_with("vap/request/") {
                                    // TODO: Make sure only the same skill is asking for it.
                                    respond(request.response, ResponseType::Valid, vec![])
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
                            match read_payload(&request.message.payload, request.response) {
                                Ok::<(MsgConnect,_),_>((p, resp)) => {
                                    if !current_skills.lock().unwrap().contains_key(&p.id) && p.vap_version == VAP_VERSION {
                                        let (sender, receiver) = oneshot::channel();
                                        let skill_id = p.id.clone();
                                        in_send.send((SkillRegisterMessage::Connect(p), sender)).await.unwrap();
                                        
                                        wait_response(receiver, resp, |r| {
                                            let code = r.status as u16;
                                            // If it is regarded as "OK"
                                            if (200..=299).contains(&code) {
                                                current_skills.lock().unwrap().insert(skill_id,());
                                            }
                                        }).await
                                    }
                                    else {
                                        respond(resp, ResponseType::BadRequest, vec![])
                                    }
                                }
                                Err(r) => {
                                    r
                                }
                            }
                        }

                        "vap/skillRegistry/registerIntents" => {
                            handle_msg(
                                request,
                                &mut in_send,
                                |p: &MsgRegisterIntents|current_skills.lock().unwrap().contains_key(&p.skill_id),
                                |p|{SkillRegisterMessage::RegisterIntents(p)}
                            ).await
                        }

                        "vap/skillRegistry/notification" => {
                            
                            match read_payload(&request.message.payload, request.response) {
                                Ok::<(MsgNotification,_),_>((msg, resp)) => {
                                    let mut standalone = vec![];
                                    let mut resolutions = vec![];

                                    enum RequestResolution {
                                        Done(msg_notification_response::Data),
                                        InProcess((RequestId, oneshot::Receiver<RequestResponse>))
                                    }

                                    let skill_id = msg.skill_id;

                                    for a in msg.data {
                                        match a {
                                            msg_notification::Data::CanYouAnswer{request_id, confidence} => {
                                                fn can_you_answer_done(response: coap_lite::ResponseType, id: RequestId) -> RequestResolution {
                                                    RequestResolution::Done(msg_notification_response::Data::CanYouAnswer {
                                                        code: response as u16,
                                                        request_id: id
                                                    })
                                                }

                                                let resol= match pending_can_you.lock().await.remove(&request_id) {
                                                    Some(pending_sender) => {
                                                        pending_sender.send(confidence).unwrap();
                                                        can_you_answer_done(coap_lite::ResponseType::Valid, request_id)
                                                        
                                                    }
                                                    None => {
                                                        can_you_answer_done(coap_lite::ResponseType::BadRequest, request_id)
                                                    }
                                                };

                                                resolutions.push(resol)
                                            }
                                            msg_notification::Data::Requested {request_id, capabilities} => {
                                                fn requested_done(response: coap_lite::ResponseType, id: RequestId) -> RequestResolution {
                                                    RequestResolution::Done(msg_notification_response::Data::Requested {
                                                        code: response as u16,
                                                        request_id: id
                                                    })
                                                }

                                                let resol = match pending_requests.lock().await.remove(&request_id) {
                                                    Some(pending_sender) => {

                                                        let (sender, receiver) = oneshot::channel();
                                                        pending_sender.send((capabilities.clone(), sender)).unwrap();
                                                        RequestResolution::InProcess((request_id, receiver))
                                                    }
                                                    None => {
                                                        requested_done(coap_lite::ResponseType::BadRequest, request_id)
                                                    }
                                                };

                                                resolutions.push(resol)
                                            }
                                            msg_notification::Data::StandAlone{client_id, capabilities} => {
                                                standalone.push(NotificationData {client_id, capabilities});
                                            }
                                        }
                                    }

                                    let mut futures = vec![];
                                    let mut other_res = vec![];
                                    for resolution in resolutions {
                                        match resolution {
                                            RequestResolution::Done(data) => {
                                                other_res.push(data);
                                            }
                                            RequestResolution::InProcess(receiver) => {
                                                futures.push(receiver)                                                
                                            }
                                        }
                                    }
                                    let (request_ids, futures) = futures.into_iter().unzip::<_,_,Vec<_>, Vec<_>>();
                                    let futs = join_all(futures);                               

                                    if !standalone.is_empty() {
                                        let send_standalone = async {
                                            let (sender, receiver) = oneshot::channel();
                                            in_send.send((SkillRegisterMessage::Notification(Notification {
                                                skill_id: skill_id.clone(),
                                                data: standalone,
                                            }), sender)).await.unwrap();
                                            wait_response(receiver, resp, |_|{}).await
                                        };

                                        // TODO: Any result that is not standalone is ignored right now (though it is processed)
                                        join(send_standalone, futs).await.0

                                    }
                                    else {
                                        let res = futs.await.into_iter()
                                            .map(|r|r.unwrap())
                                            .zip(request_ids)
                                            .map(|(n, request_id)|msg_notification_response::Data::Requested {
                                                code: n.code,
                                                request_id
                                            });
                                        other_res.extend(res);
                                            
                                        resp.map(|mut r| {
                                            let payload = to_vec_named(&MsgNotificationResponse {
                                                data: other_res
                                            }).unwrap();
                                            
                                            r.set_status(coap_lite::ResponseType::Valid);
                                            r.message.payload = payload;
                                            r
                                        })
                                    }
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
                    let path = request.get_path();
                    const BASE_SKILLS_PATH: &str = "vap/skillRegistry/skills/";
                    if path.starts_with("vap/skillRegistry/skills/") {
                        let id = &path[BASE_SKILLS_PATH.len()..];

                        match read_payload(&request.message.payload, request.response) {
                            Ok::<(MsgSkillClose, _), _>((p, resp)) => {
                                if current_skills.lock().unwrap().contains_key(id) {
                                    let (sender, receiver) = oneshot::channel();
                                    in_send.send((SkillRegisterMessage::Close(p), sender)).await.unwrap();
                                    wait_response(receiver, resp, |_|{}).await
                                }
                                else {
                                    respond(resp, ResponseType::BadRequest, vec![])
                                }
                            }
                            Err(r) => {
                                r
                            }
                        }
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
        server.run(|request| {    
            perform(
                request,
                self.in_send.clone(),
                &self.pending_requests,
                &self.pending_can_you,
                self.current_skills.clone()
            )
        }).await.unwrap();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NotificationResponse {
    pub client_id: String,
    pub code: u16
}

#[derive(Debug, Clone)]
pub struct RequestResponse {
    pub code: u16
}

pub struct SkillRegisterOut {
    client: CoAPClient,
    pending_requests: SharedPending<(Vec<PlainCapability>, oneshot::Sender<RequestResponse>)>,
    pending_can_you: SharedPending<f32>,
    next_request: RefCell<RequestId>
}

impl SkillRegisterOut {
    pub async fn skills_answerable(&self, ids: &[String], request: RequestData, client: ClientData) -> Vec<MsgNotification> {
        // TODO: Respond to the notification
        async fn send_msg(
            coap_self: &CoAPClient,
            id: &str,
            request: RequestData,
            request_id: RequestId,
            client: ClientData,
            pending_can_you: &SharedPending<f32>,
        ) -> Result<MsgNotification, Error> {
            let msg = MsgSkillRequest {client, request_id, request};
            let data = rmp_serde::to_vec(&msg).unwrap();
            let path = format!("vap/skillRegistry/skills/{}", id);
            let resp = coap_self.request_path(&path, Method::Get, Some(data), None).unwrap();
            
            assert_eq!(resp.get_status(), &coap_lite::ResponseType::Valid);

            let (sender, receiver) = oneshot::channel();
            pending_can_you.lock().await.insert(request_id, sender);
            let a = receiver.await.unwrap();

            Ok(MsgNotification{
                skill_id: id.to_string(),
                data: vec![msg_notification::Data::CanYouAnswer{request_id, confidence: a}]
            })
        }

        let mut answers = Vec::new();
        for id in ids {
            match send_msg(&self.client, id, request.clone(), self.get_id(), client.clone(), &self.pending_can_you).await {
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

    pub async fn activate_skill(&self, name: String, mut msg: MsgSkillRequest) -> Result<(Vec<PlainCapability>, oneshot::Sender<RequestResponse>), Error> {
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
//! The reference implementation of the VAP skill register.

use std::cell::RefCell;
use std::thread;
use std::{io::Cursor, collections::HashMap};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex as SyncMutex, Barrier};

use coap_lite::{RequestType as Method, CoapRequest, CoapResponse};
use coap::{CoAPClient, Server};
use futures::future::{join, join_all};
use futures::{channel::{mpsc, oneshot}, StreamExt, SinkExt, lock::Mutex};
use rmp_serde::{from_read, to_vec_named};
use serde::de::DeserializeOwned;
use thiserror::Error;
use tokio::runtime::Runtime;
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

/// VAP version implemented by this crate
pub const VAP_VERSION: &str = "Alpha";
/// The name used to refer to the skill register itself
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

/// Will handle incoming and outgoing messages to and from the skills, also
/// keeps account of the skills registered on the system.
pub struct SkillRegister {
    ip_address: String,
    in_send: mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>,
    pending_requests: SharedPending<(Vec<PlainCapability>, oneshot::Sender<RequestResponse>)>,
    pending_can_you: SharedPending<f32>,
    current_skills: Arc<SyncMutex<HashMap<String, ()>>>,
    barrier: Arc<Barrier>,
    _clnt_thrd: thread::JoinHandle<()>,
    self_send: mpsc::Sender<(String, Vec<u8>)>,
}

/// A notification received from a skill, can contain data for different VAP clients
pub struct Notification {
    pub skill_id: String,
    pub data: Vec<NotificationData>
}

/// A message from a VAP skill to a VAP client
#[derive(Debug, Clone)]
pub struct NotificationData {
    pub client_id: String,
    pub capabilities: Vec<structures::PlainCapability>
}

/// A message received from a skill
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
    
    /// Creates a new skill register, the skill register is divided into three parts:
    /// 1. The skill register task, which will handle everything behind the scenes, you just need to await on run().
    /// 2. The skill stream, which will receive all the messages from the skills.
    /// 3. The skill out, which you can use to send messages to the skills.
    /// # Arguments
    /// * `port` - The port for the skill register to listen CoAP messages on.    
    pub fn new(port: u16) -> Result<(Self, SkillRegisterStream, SkillRegisterOut), Error> {   
        let (in_send, in_recv) = mpsc::channel(20);
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let pending_can_you = Arc::new(Mutex::new(HashMap::new()));
        let barrier = Arc::new(Barrier::new(2));

        let (self_send, mut self_recv) = mpsc::channel(20);
        let barrier2 = barrier.clone();
        let _clnt_thrd = thread::spawn(move || {
            // In Linux we need a second thread to to send to ourselves, otherwise
            // what would be a non-blocking operation, tries to block, which
            // returns an error.
            Runtime::new().unwrap().block_on(async move {
                let ip_address = format!("127.0.0.1:{}", port);
                let client = CoAPClient::new(&ip_address).unwrap();
                barrier2.wait(); // Make sure we are not sending anything before the server is ready

                loop {
                    let (name, data): (String, _) = self_recv.next().await.unwrap();
                    let resp = client.request_path(&format!("vap/skillRegistry/skills/{}", name), Method::Put, Some(data), None).unwrap();
                    assert_eq!(resp.get_status(), &coap_lite::ResponseType::Valid);
                }
            });
        });

        Ok((
            SkillRegister {
                ip_address: format!("127.0.0.1:{}", port),
                in_send,
                pending_requests: pending_requests.clone(),
                pending_can_you: pending_can_you.clone(),
                current_skills: Arc::new(SyncMutex::new(HashMap::new())),
                barrier,
                _clnt_thrd,
                self_send: self_send.clone()
            },

            SkillRegisterStream {
                stream_in: in_recv,
            },

            SkillRegisterOut {pending_requests, self_send,
                pending_can_you, next_request: RefCell::new(0)}
        ))
    }
 
    /// Call this function and await it for the rest of the program, this handles
    /// sending and receiving messages from the skills. Stopping this means no more
    /// communication, and even dropped channels.
    pub async fn run(self) -> Result<(), Error>  {
        async fn perform(
            request: CoapRequest<SocketAddr>,
            mut in_send: mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>,
            pending_requests: &SharedPending<(Vec<PlainCapability>, oneshot::Sender<RequestResponse>)>,
            pending_can_you: &SharedPending<f32>,
            current_skills: Arc<SyncMutex<HashMap<String, ()>>>,
            mut self_send: mpsc::Sender<(String, Vec<u8>)>
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
                                    SkillRegisterMessage::Query
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
                                            // If it is regarded as "OK"
                                            if [
                                                ResponseType::Created,ResponseType::Deleted,
                                                ResponseType::Valid,
                                                ResponseType::Changed,
                                                ResponseType::Content,
                                                ResponseType::Continue
                                                ].contains(&r.status) {
                                                
                                                // We need to register the skill inside the CoAP server
                                                self_send.try_send((skill_id.clone(), vec![])).unwrap();
                                                current_skills.lock().unwrap().insert(skill_id.clone(),());
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
                                SkillRegisterMessage::RegisterIntents
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

        // The server is ready, we can start to send requests to itself
        // This is added because of an issue with the client trying to acces the
        // server too soon on Linux.
        self.barrier.wait(); 
        server.run(|request| {    
            perform(
                request,
                self.in_send.clone(),
                &self.pending_requests,
                &self.pending_can_you,
                self.current_skills.clone(),
                self.self_send.clone()
            )
        }).await.unwrap();
        Ok(())
    }
}

/// Whether a notification could be handled or some problem arised
#[derive(Debug, Clone)]
pub struct NotificationResponse {
    pub client_id: String,
    /// Response code of the request. Use coap codes (e.g: 200 for success, 404 for not found)
    pub code: u16 
}

/// Whether a notification could be handled or some problem arised
#[derive(Debug, Clone)]
pub struct RequestResponse {
    /// Use coap codes (e.g: 200 for success, 404 for not found)
    pub code: u16 
}

/// An object for sending messages to skills
pub struct SkillRegisterOut {
    pending_requests: SharedPending<(Vec<PlainCapability>, oneshot::Sender<RequestResponse>)>,
    pending_can_you: SharedPending<f32>,
    next_request: RefCell<RequestId>,
    self_send: mpsc::Sender<(String, Vec<u8>)>,
}

impl SkillRegisterOut {
    /// Returns how confident skills registered for some request are in being able to handle it
    pub async fn skills_answerable(&mut self, ids: &[String], request: RequestData, client: ClientData) -> Vec<MsgNotification> {
        // TODO: Respond to the notification
        async fn send_msg(
            self_send: &mut mpsc::Sender<(String, Vec<u8>)>,
            id: &str,
            request: RequestData,
            request_id: RequestId,
            client: ClientData,
            pending_can_you: &SharedPending<f32>
        ) -> Result<MsgNotification, Error> {
            let msg = MsgSkillRequest {client, request_id, request};
            let data = rmp_serde::to_vec(&msg).unwrap();
            self_send.send((id.into(), data)).await.unwrap();

            let (sender, receiver) = oneshot::channel();
            pending_can_you.lock().await.insert(request_id, sender);
            let a = receiver.await.unwrap();

            Ok(MsgNotification{
                skill_id: id.to_string(),
                data: vec![msg_notification::Data::CanYouAnswer{request_id, confidence: a}]
            })
        }

        let mut answers = Vec::new();
        let new_id = self.get_id();
        for id in ids {
            match send_msg(&mut self.self_send, id, request.clone(), new_id, client.clone(), &self.pending_can_you).await {
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

    /// Sends a request to a skill
    pub async fn activate_skill(&mut self, name: String, mut msg: MsgSkillRequest) -> Result<(Vec<PlainCapability>, oneshot::Sender<RequestResponse>), Error> {
        // TODO: Respond to the notification
        let req_id = self.get_id();
        msg.request_id = req_id;
        let (sender,receiver) = oneshot::channel();
        let data = rmp_serde::to_vec(&msg).unwrap();
        self.self_send.send((name, data)).await.unwrap();

        self.pending_requests.lock().await.insert(req_id, sender);
        
        let resp_data = receiver.await.unwrap();
        Ok(resp_data)
    }
}

/// An object that will receive notifications from skills
pub struct SkillRegisterStream {
    stream_in: mpsc::Receiver<(SkillRegisterMessage, oneshot::Sender<Response>)>,
}

impl SkillRegisterStream {
    /// Await this on a loop to get notifications from skills
    /// # Examples
    /// ```
    /// loop {
    ///     let (msg, response) = skill_register_stream.recv().await.unwrap();
    /// }
    ///```
    pub async fn recv(&mut self) -> Result<(SkillRegisterMessage, oneshot::Sender<Response>), Error> {
        Ok(self.stream_in.next().await.unwrap())
    }
}
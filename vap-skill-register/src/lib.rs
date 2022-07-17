//! The reference implementation of the VAP skill register.

mod method_handlers;
mod vars;

use std::cell::RefCell;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Barrier, Mutex as SyncMutex};
use std::thread;

use coap::{CoAPClient, Server};
use coap_lite::{CoapRequest, CoapResponse, RequestType as Method};
use futures::{
    channel::{mpsc, oneshot},
    lock::Mutex,
    SinkExt, StreamExt,
};
use thiserror::Error;
use tokio::runtime::Runtime;
use vap_common_skill::structures::msg_skill_request::{ClientData, RequestData};
use vap_common_skill::structures::*;

pub use coap_lite::ResponseType;
pub use vap_common_skill::structures;
pub use vars::{SYSTEM_SELF_ID, VAP_VERSION};

type RequestId = u64;
type SharedPending<D> = Arc<Mutex<HashMap<RequestId, oneshot::Sender<D>>>>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("A Oneshot channel was closed")]
    ClosedChannel,
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
    pub data: Vec<NotificationData>,
}

/// A message from a VAP skill to a VAP client
#[derive(Debug, Clone)]
pub struct NotificationData {
    pub client_id: String,
    pub capabilities: Vec<structures::PlainCapability>,
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
    resp.map(|mut c| {
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
                let mut client = CoAPClient::new(&ip_address).unwrap();
                barrier2.wait(); // Make sure we are not sending anything before the server is ready

                loop {
                    let (name, data): (String, Vec<u8>) = self_recv.next().await.unwrap();
                    println!("Size: {}", data.len());
                    let resp = client
                        .request_path(
                            &format!("vap/skillRegistry/skills/{}", name),
                            Method::Put,
                            Some(data),
                            None,
                        )
                        .unwrap();
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
                self_send: self_send.clone(),
            },
            SkillRegisterStream { stream_in: in_recv },
            SkillRegisterOut {
                pending_requests,
                self_send,
                pending_can_you,
                next_request: RefCell::new(0),
            },
        ))
    }

    /// Call this function and await it for the rest of the program, this handles
    /// sending and receiving messages from the skills. Stopping this means no more
    /// communication, and even dropped channels.
    pub async fn run(self) -> Result<(), Error> {
        async fn perform(
            request: CoapRequest<SocketAddr>,
            mut in_send: mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>,
            pending_requests: &SharedPending<(
                Vec<PlainCapability>,
                oneshot::Sender<RequestResponse>,
            )>,
            pending_can_you: &SharedPending<f32>,
            current_skills: Arc<SyncMutex<HashMap<String, ()>>>,
            mut self_send: mpsc::Sender<(String, Vec<u8>)>,
        ) -> Option<CoapResponse> {
            match *request.get_method() {
                Method::Get => method_handlers::on_get(request, &mut in_send, current_skills).await,
                Method::Post => {
                    method_handlers::on_post(
                        request,
                        &mut in_send,
                        &mut self_send,
                        &current_skills,
                        pending_can_you,
                        pending_requests,
                    )
                    .await
                }
                Method::Delete => {
                    method_handlers::on_delete(request, &mut in_send, current_skills).await
                }
                Method::Put =>
                // Puts are needed so that an observe update is produced
                {
                    if request.get_path().starts_with("vap/skillRegistry/skill") {
                        respond(request.response, coap_lite::ResponseType::Valid, vec![])
                    }
                    else {
                        respond(request.response, coap_lite::ResponseType::MethodNotAllowed, vec![])
                    }
                }

                _ => {
                    println!("request by other method");
                    respond(
                        request.response,
                        coap_lite::ResponseType::MethodNotAllowed,
                        vec![],
                    )
                }
            }
        }

        let mut server = Server::new(&self.ip_address).unwrap();
        server.enable_all_coap(0);

        // The server is ready, we can start to send requests to itself
        // This is added because of an issue with the client trying to acces the
        // server too soon on Linux.
        self.barrier.wait();
        server
            .run(|request| {
                perform(
                    request,
                    self.in_send.clone(),
                    &self.pending_requests,
                    &self.pending_can_you,
                    self.current_skills.clone(),
                    self.self_send.clone(),
                )
            })
            .await
            .unwrap();
        Ok(())
    }
}

/// Whether a notification could be handled or some problem arised
#[derive(Debug, Clone)]
pub struct NotificationResponse {
    pub client_id: String,
    /// Response code of the request. Use coap codes (e.g: 200 for success, 404 for not found)
    pub code: u16,
}

/// Whether a notification could be handled or some problem arised
#[derive(Debug, Clone)]
pub struct RequestResponse {
    /// Use coap codes (e.g: 200 for success, 404 for not found)
    pub code: u16,
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
    pub async fn skills_answerable(
        &mut self,
        ids: &[String],
        request: RequestData,
        client: ClientData,
    ) -> Vec<MsgNotification> {
        // TODO: Respond to the notification
        async fn send_msg(
            self_send: &mut mpsc::Sender<(String, Vec<u8>)>,
            id: &str,
            request: RequestData,
            request_id: RequestId,
            client: ClientData,
            pending_can_you: &SharedPending<f32>,
        ) -> Result<MsgNotification, Error> {
            let msg = MsgSkillRequest {
                client,
                request_id,
                request,
            };
            let data = rmp_serde::to_vec(&msg).unwrap();
            self_send.send((id.into(), data)).await.unwrap();

            let (sender, receiver) = oneshot::channel();
            pending_can_you.lock().await.insert(request_id, sender);
            let a = receiver.await.unwrap();

            Ok(MsgNotification {
                skill_id: id.to_string(),
                data: vec![msg_notification::Data::CanYouAnswer {
                    request_id,
                    confidence: a,
                }],
            })
        }

        let mut answers = Vec::new();
        let new_id = self.get_id();
        for id in ids {
            match send_msg(
                &mut self.self_send,
                id,
                request.clone(),
                new_id,
                client.clone(),
                &self.pending_can_you,
            )
            .await
            {
                Ok(resp) => {
                    println!("We received the notification: {:?}", resp);
                    answers.push(resp);
                }
                Err(e) => {
                    // TODO: What to do with the errors?
                    println!("Error receiving notifications: {:?}", e);
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
    pub async fn activate_skill(
        &mut self,
        name: String,
        mut msg: MsgSkillRequest,
    ) -> Result<(Vec<PlainCapability>, oneshot::Sender<RequestResponse>), Error> {
        // TODO: Respond to the notification
        let req_id = self.get_id();
        msg.request_id = req_id;
        let (sender, receiver) = oneshot::channel();
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
    pub async fn recv(
        &mut self,
    ) -> Result<(SkillRegisterMessage, oneshot::Sender<Response>), Error> {
        Ok(self.stream_in.next().await.unwrap())
    }
}

mod load;

use std::{io::Cursor, path::Path};

use coap::CoAPClient;
use coap_lite::{MessageClass, RequestType as Method, ResponseType};
use fluent_langneg::negotiate_languages;
use futures::channel::mpsc;
use log::warn;
use serde::Serialize;
use thiserror::Error;
use unic_langid::LanguageIdentifier;
use vap_common_skill::structures::{msg_notification::Data, msg_query::QueryData, *, msg_skill_request::RequestSlot};

pub use vap_common_skill::structures::{msg_skill_request::RequestDataKind, PlainCapability};

/// The skill itself, use this to communicate with the registry.
pub struct Skill {
    client: CoAPClient,
    id: String,
    langs: Vec<LanguageIdentifier>,
    sender: mpsc::Sender<SkillRequest>,
}

impl Skill {
    fn get_address() -> String {
        const PORT: u16 = 5683;
        format!("127.0.0.1:{}", PORT)
    }

    /// Creates a new skill, will also connect to the skill registry and register
    /// itself so that it receives requests. Returns both itself and a channel
    /// that you will use to receive events. This follows RAII and as as soon as
    /// it is dropped will disconnect from the skill registry.
    /// 
    /// # Arguments
    /// 
    /// * `name` - A human-readable name for this skill
    /// * `id` -  This skill id like 'com.my_company.my_skill'
    /// * `intents` - Where are the skills stored
    /// 
    pub fn new<S1, S2, P>(name: S1, id: S2, intents: P) -> Result<(Self, SkillIn)>
    where
        S1: Into<String>,
        S2: Into<String>,
        P: AsRef<Path> + Clone,
    {
        let id_str = id.into();
        let payload = rmp_serde::to_vec_named(&MsgConnect {
            id: id_str.clone(),
            name: name.into(),
            vap_version: "Alpha".into(),
            unique_authentication_token: Some("".into()),
        })
        .expect("Failed to make initial payload, report this");

        let client = CoAPClient::new(Self::get_address())?;
        let resp = client.request_path(
            "vap/skillRegistry/connect",
            Method::Post,
            Some(payload),
            None,
        )?;

        match resp.message.header.code {
            MessageClass::Response(ResponseType::Created) => {
                let payload: MsgConnectResponse =
                    rmp_serde::from_read(Cursor::new(resp.message.payload)).unwrap();
                let (sender, receiver) = mpsc::channel(10);

                let mut skill = Self {
                    client,
                    id: id_str,
                    langs: payload.langs.into_iter().map(|l| l.into()).collect(),
                    sender,
                };

                skill.register_intents(intents)?;
                skill.register()?;

                Ok((skill, receiver))
            }
            _ => {
                panic!("ERROR")
            }
        }
    }

    fn send_message<T: Serialize>(
        &self,
        method: Method,
        path: &str,
        data: T,
    ) -> Result<(ResponseType, Vec<u8>)> {
        println!("Sending message");
        let d = rmp_serde::to_vec_named(&data).expect("Failed to encode message, report this");
        let resp = self
            .client
            .request_path(path, method, Some(d), None)
            .unwrap();
        println!("Received!");

        Ok((
            extract_type(resp.message.header.code), 
            resp.message.payload
        ))
    }

    fn send_message_no_payload(&self, method: Method, path: &str) -> ResponseType {
        extract_type(
            self.client
                .request_path(path, method, None, None)
                .unwrap()
                .message
                .header
                .code,
        )
    }

    pub fn register_intents<P>(&mut self, intents: P) -> Result<()>
    where
        P: AsRef<Path> + Clone,
    {
        let langs = load::list_langs(intents.clone());
        println!("payload langs: {:?}", &self.langs);
        let langs = negotiate_languages(
            &self.langs,
            &langs,
            None,
            fluent_langneg::NegotiationStrategy::Matching,
        );

        let nlu_data = load::load_intents(&langs, intents);
        println!("INtents: {:?}", nlu_data);

        match self.send_message(
            Method::Post,
            "vap/skillRegistry/registerIntents",
            MsgRegisterIntents {
                skill_id: self.id.clone(),
                nlu_data,
            },
        )? {
            (ResponseType::Created, _) => Ok(()),
            _ => Err(Error::Unknown),
        }
    }

    fn close(&mut self) -> Result<()> {
        if self.send_message_no_payload(
            Method::Delete,
            &format!("vap/skillRegistry/skills/{}", &self.id),
        ) != ResponseType::Deleted
        {
            Err(Error::Unknown)
        } else {
            Ok(())
        }
    }

    /// Send a standalone notification to some ID (a client or the system itself)
    pub fn notify(
        &mut self,
        client_id: String,
        capabilities: Vec<PlainCapability>,
    ) -> Result<MsgNotificationResponse> {
        self.notify_multiple(vec![Data::StandAlone {
            client_id,
            capabilities,
        }])
    }

    /// Send notifications (of any type) to several clients
    pub fn notify_multiple(&mut self, data: Vec<Data>) -> Result<MsgNotificationResponse> {
        println!("Send answer");
        match self.send_message(
            Method::Post,
            "vap/skillRegistry/notification",
            MsgNotification {
                skill_id: self.id.clone(),
                data,
            },
        )? {
            (ResponseType::Content, d) => Ok(rmp_serde::from_read(Cursor::new(d))
                .expect("Failed to create MsgNotification, report this")),
            _ => Err(Error::Unknown),
        }
    }

    /// Send queries to any number of IDs (clients or the system itself).
    pub fn query(&mut self, data: Vec<QueryData>) -> Result<MsgQueryResponse> {
        match self.send_message(
            Method::Get,
            "vap/skillRegistry/query",
            MsgQuery {
                skill_id: self.id.clone(),
                data,
            },
        )? {
            (ResponseType::Content, d) => Ok(rmp_serde::from_read(Cursor::new(d))
                .expect("Failed to create MsgQuery, report this")),
            (ResponseType::BadRequest, _) => Err(Error::BadRequest),

            _ => Err(Error::Unknown),
        }
    }

    fn register(&mut self) -> Result<()> {
        let mut sender = self.sender.clone();
        self.client
            .observe(
                &format!("vap/skillRegistry/skills/{}", &self.id),
                move |m| {
                    println!("Oberseve returned something!!!");
                    println!("{:?}", m);
                    if !m.payload.is_empty()
                        && m.header.code == MessageClass::Response(ResponseType::Content)
                    {
                        println!("Msg:  {:?}", debug_msg_pack(&m.payload));

                        match rmp_serde::from_read::<_, MsgSkillRequest>(Cursor::new(m.payload)) {
                            Ok(payload) => {
                                sender.try_send(payload.into()).unwrap();
                            }
                            Err(e) => {
                                warn!("Received a bad msgpack message, will be ignored: {}", e);
                            }
                        }
                    }
                },
            )?;
            Ok(())
    }

    /// Answer an incoming request
    pub fn answer(
        &mut self,
        req: &SkillRequest,
        capabilities: Vec<PlainCapability>,
    ) -> Result<()> {
        self.notify_multiple(vec![Data::Requested {
            request_id: req.request_id,
            capabilities,
        }])?;

        Ok(())
    }
}

impl Drop for Skill {
    fn drop(&mut self) {
        self.close().unwrap();
    }
}

fn debug_msg_pack(payload: &[u8]) -> String {
    let v: Value = rmp_serde::from_read(Cursor::new(payload.to_vec())).unwrap();
    v.to_string()
}

fn extract_type(code: MessageClass) -> ResponseType {
    if let MessageClass::Response(c) = code {
        c
    } else {
        panic!("Should be a response")
    }
}

#[derive(Clone, Debug)]
pub struct SkillRequest {
    pub request_id: u64,
    pub client: msg_skill_request::ClientData,
    pub request: Request
}

#[derive(Clone, Debug)]
pub enum Request {
    Intent(String, RequestData),
    Event(String, RequestData),
    CanAnswer(String, RequestData)
}

#[derive(Clone, Debug)]
pub enum RequestStr<'a> {
    Intent(&'a str, &'a RequestData),
    Event(&'a str, &'a RequestData),
    CanAnswer(&'a str, &'a RequestData)
}

impl Request {
    pub fn as_str(& self) -> RequestStr<'_>  {
        match &self {
            Request::Intent(s, d) => RequestStr::Intent(s.as_str(), d),
            Request::Event(s, d) => RequestStr::Event(s.as_str(), d),
            Request::CanAnswer(s, d) => RequestStr::CanAnswer(s.as_str(), d)
        }
    }
}

#[derive(Clone, Debug)]
pub struct RequestData {
    pub locale: String,
    pub slots: Vec<RequestSlot>
}

impl From<MsgSkillRequest> for SkillRequest {
    fn from(msg: MsgSkillRequest) -> Self {
        let req_data = RequestData {
            locale: msg.request.locale,
            slots: msg.request.slots
        };

        let request = match msg.request.type_ {
            RequestDataKind::CanAnswer => Request::CanAnswer(msg.request.intent, req_data),
            RequestDataKind::Event => Request::Event(msg.request.intent, req_data),
            RequestDataKind::Intent => Request::Intent(msg.request.intent, req_data)
        };

        Self { 
            request_id: msg.request_id,
            client: msg.client,
            request}
    }
} 

type SkillIn = mpsc::Receiver<SkillRequest>;

type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO")]
    IO(#[from] std::io::Error),

    #[error("The data sent had a wrong format or didn't meet the VAP rules")]
    BadRequest,

    #[error("We got an error, but we don't know why")]
    Unknown,
}

#[cfg(test)]
mod tests {
    use crate::Skill;
    use futures::StreamExt;

    #[tokio::test]
    async fn it_works() {
        let (skill, mut skill_in) = Skill::new("Test", "com.example.test", "assets").unwrap();
        loop {
            let req = skill_in.next().await.unwrap();
        }
    }
}

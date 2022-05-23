mod load;

use std::{io::Cursor, path::Path, collections::HashMap};

use coap::CoAPClient;
use coap_lite::{MessageClass, RequestType as Method, ResponseType};
use fluent_langneg::negotiate_languages;
use futures::channel::mpsc;
use serde::Serialize;
use thiserror::Error;
use unic_langid::LanguageIdentifier;
use vap_common_skill::structures::{
    msg_notification::Data,
    msg_query::QueryData,
    *,
};

pub use vap_common_skill::structures::{PlainCapability, msg_skill_request::RequestDataKind};

pub struct Skill {
    client: CoAPClient,
    id: String,
    langs: Vec<LanguageIdentifier>,
    sender: mpsc::Sender<MsgSkillRequest>,
}

impl Skill {
    fn get_address() -> String {
        const PORT: u16 = 5683;
        format!("127.0.0.1:{}", PORT)
    }

    pub fn new<S1, S2, P>(name: S1, id: S2, intents: P) -> (Self, SkillIn)
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

        let client = CoAPClient::new(Self::get_address()).unwrap();
        let resp = client
            .request_path(
                "vap/skillRegistry/connect",
                Method::Post,
                Some(payload),
                None,
            )
            .unwrap();

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

                skill.register_intents(intents);
                skill.register();

                (skill, receiver)
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
    ) -> (ResponseType, Vec<u8>) {
        println!("Sending message");
        let d = rmp_serde::to_vec_named(&data).unwrap();
        let resp = self
            .client
            .request_path(path, method, Some(d), None)
            .unwrap();
        println!("Received!");
        if let MessageClass::Response(c) = resp.message.header.code {
            (c, resp.message.payload)
        } else {
            panic!("Should be a response")
        }
    }

    fn send_message_no_payload(&self, method: Method, path: &str) -> ResponseType {
        let resp = self.client.request_path(path, method, None, None).unwrap();
        if let MessageClass::Response(c) = resp.message.header.code {
            c
        } else {
            panic!("Should be a response")
        }
    }

    pub fn register_intents<P>(&mut self, intents: P)
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
        ) {
            (ResponseType::Created, _) => {}
            _ => panic!("Response not good"),
        }
    }

    pub fn close(&mut self) {
        if self.send_message_no_payload(
            Method::Delete,
            &format!("vap/skillRegistry/skills/{}", &self.id),
        ) != ResponseType::Deleted
        {
            panic!("Response not good")
        }
    }

    pub fn notify(
        &mut self,
        client_id: String,
        capabilities: Vec<PlainCapability>,
    ) -> MsgNotificationResponse {
        self.notify_multiple(vec![Data::StandAlone {
            client_id,
            capabilities,
        }])
    }

    pub fn notify_multiple(&mut self, data: Vec<Data>) -> MsgNotificationResponse {
        println!("Send answer");
        match self.send_message(
            Method::Post,
            "vap/skillRegistry/notification",
            MsgNotification {
                skill_id: self.id.clone(),
                data,
            },
        ) {
            (ResponseType::Content, d) => {
                Oberseve returned something!!!id.clone(),
                data,
            },
        ) {
            (ResponseType::Content, d) => rmp_serde::from_read(Cursor::new(d)).unwrap(),

            _ => panic!("Failed to query data"),
        }
    }

    pub fn register(&mut self) {
        let mut sender = self.sender.clone();
        self.client
            .observe(
                &format!("vap/skillRegistry/skills/{}", &self.id),
                move |m| {
                    println!("Oberseve returned something!!!");
                    println!("{:?}", m);
                    if !m.payload.is_empty() && m.header.code == MessageClass::Response(ResponseType::Content) {
                        println!("Msg:  {:?}", debug_msg_pack(&m.payload));

                        let payload: MsgSkillRequest =
                            rmp_serde::from_read(Cursor::new(m.payload)).unwrap();

                        sender
                            .try_send(payload)
                            .unwrap();
                    }
                },
            )
            .unwrap();
    }

    pub fn answer(&mut self, req: &MsgSkillRequest, capabilities: Vec<PlainCapability>) {
        self.notify_multiple(vec![
            Data::Requested {
                request_id: req.request_id,
                capabilities
            }
        ]);
    }
}

impl Drop for Skill {
    fn drop(&mut self) {
        self.close();
    }
}

fn debug_msg_pack(payload: &[u8]) -> String {
    let v: Value = rmp_serde::from_read(Cursor::new(payload.to_vec())).unwrap();
    v.to_string()
}

type SkillIn = mpsc::Receiver<MsgSkillRequest>;

type Result<T> = core::result::Result<T, Error>;
pub enum Error {
    
}

#[cfg(test)]
mod tests {
    use crate::Skill;
    use futures::StreamExt;

    #[tokio::test]
    async fn it_works() {
        let (mut skill, mut skill_in) = Skill::new("Test", "com.example.test", "assets");
        loop {
            let req = skill_in.next().await.unwrap();
        }
    }
}

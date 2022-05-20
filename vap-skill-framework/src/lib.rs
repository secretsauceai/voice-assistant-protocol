mod load;

use std::{io::Cursor, path::Path};

use coap::CoAPClient;
use coap_lite::{MessageClass, RequestType as Method, ResponseType};
use fluent_langneg::negotiate_languages;
use futures::channel::mpsc;
use serde::Serialize;
use unic_langid::LanguageIdentifier;
use vap_common_skill::structures::{
    msg_notification::Data,
    msg_query::QueryData,
    msg_skill_request::{ClientData, RequestData},
    *,
};

pub use vap_common_skill::structures::msg_skill_request::RequestDataKind;

pub struct Skill {
    client: CoAPClient,
    id: String,
    langs: Vec<LanguageIdentifier>,
    sender: mpsc::Sender<RegistryRequest>,
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
        .unwrap();

        let client = CoAPClient::new(Self::get_address()).unwrap();
        let resp = client
            .request_path(
                "vap/skillRegistry/connect",
                Method::Put,
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
        let d = rmp_serde::to_vec_named(&data).unwrap();
        let resp = self
            .client
            .request_path(path, method, Some(d), None)
            .unwrap();
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
        let langs = negotiate_languages(
            &self.langs,
            &langs,
            None,
            fluent_langneg::NegotiationStrategy::Matching,
        );

        let nlu_data = load::load_intents(&langs, intents);

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
        match self.send_message(
            Method::Post,
            "vap/skillRegistry/notification",
            MsgNotification {
                skill_id: self.id.clone(),
                data,
            },
        ) {
            (ResponseType::Content, d) => rmp_serde::from_read(Cursor::new(d)).unwrap(),
            _ => panic!("Failed to send notification"),
        }
    }

    pub fn query(&mut self, data: Vec<QueryData>) -> MsgQueryResponse {
        match self.send_message(
            Method::Get,
            "vap/skillRegistry/query",
            MsgQuery {
                skill_id: self.id.clone(),
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
                    let payload: MsgSkillRequest =
                        rmp_serde::from_read(Cursor::new(m.payload)).unwrap();

                    sender
                        .try_send(RegistryRequest {
                            client: payload.client,
                            request: payload.request,
                        })
                        .unwrap();
                },
            )
            .unwrap();
    }
}

impl Drop for Skill {
    fn drop(&mut self) {
        self.close();
    }
}

pub struct RegistryRequest {
    pub client: ClientData,
    pub request: RequestData,
}

type SkillIn = mpsc::Receiver<RegistryRequest>;

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

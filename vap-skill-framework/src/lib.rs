use std::io::Cursor;

use coap::CoAPClient;
use coap_lite::{RequestType as Method, ResponseType, MessageClass};
use futures::channel::mpsc;
use serde::Serialize;
use unic_langid::LanguageIdentifier;
use vap_common_skill::structures::{*, msg_register_intents::NluData, msg_notification::Data, msg_query::QueryData, msg_skill_request::{RequestData, ClientData}};

pub struct Skill {
    client: CoAPClient,
    id: String,
    langs: Vec<LanguageIdentifier>,
    sender: mpsc::Sender<RegistryRequest>
}

fn from_lang_struct(lang: Language) -> LanguageIdentifier {
    LanguageIdentifier::from_parts(
        lang.language.parse().unwrap(),
        lang.extra.and_then(|e|e.parse().ok()),
        lang.country.and_then(|c|c.parse().ok()),
        &[]
    )
}

impl Skill {
    fn get_address() -> String {
        const PORT: u16 = 5683;
        format!("127.0.0.1:{}", PORT)
    }

    pub fn new<S1, S2>(name: S1, id: S2) -> (Self , SkillIn)
        where
        S1: Into<String>,
        S2: Into<String> {

        let id_str = id.into();
        let payload = rmp_serde::to_vec_named(&MsgConnect {
            id: id_str.clone(),
            name: name.into(),
            vap_version: "Alpha".into(),
            unique_authentication_token: Some("".into())
        }).unwrap();
        
        let client = CoAPClient::new(Self::get_address()).unwrap();
        let resp = client.request_path(
            "vap/skillRegistry/connect",
            Method::Put,
            Some(payload),
            None,
        ).unwrap();

        match resp.message.header.code  {
            MessageClass::Response(ResponseType::Created) => {
                let payload: MsgConnectResponse = rmp_serde::from_read(Cursor::new(resp.message.payload)).unwrap();
                let (sender, receiver) = mpsc::channel(10);
                
                (
                    Self {client, id: id_str, langs: payload.langs.into_iter().map(from_lang_struct).collect(), sender},
                    receiver
                )
            },
            _ => {panic!("ERROR")}
        }        
    }

    fn send_message<T: Serialize>(&self, method: Method, path:&str, data: T) -> (ResponseType, Vec<u8>) {
        let d = rmp_serde::to_vec_named(&data).unwrap();
        let resp = self.client.request_path(path, method, Some(d), None).unwrap();
        if let MessageClass::Response(c) = resp.message.header.code {
            (c, resp.message.payload)
        }
        else {
            panic!("Should be a response")
        }
    }

    fn send_message_no_payload(&self, method: Method, path:&str) -> ResponseType {        
        let resp = self.client.request_path(path, method, None, None).unwrap();
        if let MessageClass::Response(c) = resp.message.header.code {
            c
        }
        else {
            panic!("Should be a response")
        }
    }

    pub fn register_intents(&mut self, nlu_data: Vec<NluData>) {
        match self.send_message(
            Method::Post,
            "vap/skillRegistry/registerIntents",
            MsgRegisterIntents {
                skill_id: self.id.clone(),
                nlu_data
            }) {
                (ResponseType::Created,_) => {}
                _ => panic!("Response not good")
            }
    }

    pub fn close(&mut self) {
        if self.send_message_no_payload(
            Method::Delete,
            &format!("vap/skillRegistry/skills/{}", &self.id)
        ) != ResponseType::Deleted {
            panic!("Response not good")
        }
    }

    pub fn notify(&mut self, client_id: String, capabilities: Vec<PlainCapability>) -> MsgNotificationResponse {
        self.notify_multiple(vec![Data::StandAlone{client_id, capabilities}])
    }

    pub fn notify_multiple(&mut self, data: Vec<Data>) -> MsgNotificationResponse {
        match self.send_message(
            Method::Post,
            "vap/skillRegistry/notification",
            MsgNotification {skill_id: self.id.clone(), data}
        ) {
            (ResponseType::Content, d) => {
                rmp_serde::from_read(Cursor::new(d)).unwrap()
            }
            _ => panic!("Failed to send notification")
        }
    }

    pub fn query(&mut self, data: Vec<QueryData>) -> MsgQueryResponse {
        match self.send_message(
            Method::Get,
            "vap/skillRegistry/query",
            MsgQuery {skill_id: self.id.clone(), data}
        ) {
            (ResponseType::Content, d) => {
                rmp_serde::from_read(Cursor::new(d)).unwrap()
            }

            _ => panic!("Failed to query data")
        }
    }

    pub async fn register(&mut self) {
        let mut sender = self.sender.clone();
        self.client.observe(&format!("vap/skillRegistry/skills/{}", &self.id), move |m| {
            let payload: MsgSkillRequest = rmp_serde::from_read(Cursor::new(m.payload)).unwrap();
            
            sender.try_send(RegistryRequest {client: payload.client, request: payload.request}).unwrap();
        }).unwrap();
    }
}

pub struct RegistryRequest {
    client: ClientData,
    request: RequestData
}

type SkillIn = mpsc::Receiver<RegistryRequest>;

#[cfg(test)]
mod tests {
    use crate::Skill;

    #[test]
    fn it_works() {
        let (mut skill, mut skill_in) = Skill::new("Test", "com.example.test");
        skill.register_intents(vec![]);
        skill.register();
        skill.close();
    }
}

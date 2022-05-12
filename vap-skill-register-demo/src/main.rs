use std::{collections::HashMap, time::Duration};

use tokio::sync::oneshot;
use vap_skill_register::{
    SkillRegister, Response, ResponseType, SkillRegisterStream, SkillRegisterOut,
    SkillRegisterMessage,
    structures::{
        MsgConnectResponse, Language, MsgQueryResponse, MsgSkillRequest,
        msg_skill_request::{ClientData, RequestData, RequestDataKind},
        msg_query_response::{QueryData, QueryDataCapability}, Value,
    }
};


mod conf {
    pub const PORT: u16 = 5683;
}

struct MyData {
    name: Option<oneshot::Sender<String>>
}

impl MyData {
    async fn on_msg(&mut self, mut stream: SkillRegisterStream) -> Result<(), vap_skill_register::Error> {
        loop {
            let (msg, responder) = stream.recv().await?;
            let resp = match msg {
                SkillRegisterMessage::Connect(m) => {
                    println!("{} wants to connect", m.id);
                    self.name.take().map(|c|c.send(m.id).unwrap());
                    let data= rmp_serde::to_vec(&MsgConnectResponse {
                        unique_authentication_token: None,
                        langs: vec![
                            Language {
                                language: "en".to_string(),
                                country: Some("US".to_string()),
                                extra: None
                            },
                        ]
                    }).unwrap();
                    Response {
                        status: ResponseType::Created,
                        payload:  data
                    }
                },
                SkillRegisterMessage::RegisterIntents(m) => {
                    println!("{} wants to register this data: {:?}", m.skill_id, m.nlu_data);
                    Response {
                        status: ResponseType::Created,
                        payload:  vec![]
                    }
                },
                SkillRegisterMessage::Query(m) => {
                    println!("{} wants to query this data: {:?}", m.skill_id, m.data);

                    let data = m.data.into_iter().map(|x| {
                        let capabilities = x.capabilities.into_iter().map(|x| {
                            let (code, payload) = match x.name.as_str() {
                                "preferences" => {
                                    if let Some(what) =  x.cap_data.get(&"what".into()) {
                                        if what == &Value::String("color".into()) {
                                            let mut res = HashMap::new();
                                            res.insert("color".into(), "red".into());
                                            (205, res)
                                        }
                                        else {
                                            (400, HashMap::new())
                                        }
                                    }
                                    else {
                                        (400, HashMap::new())
                                    }
                                }
                                _ => {
                                    (400, HashMap::new())
                                }
                            };

                            QueryDataCapability {
                                name: x.name,
                                code,
                                data: payload
                            }
                        }).collect::<Vec<_>>();

                        QueryData {
                            client_id: x.client_id,
                            capabilities
                        }
                    }).collect::<Vec<_>>();
                    let payload = rmp_serde::to_vec(&MsgQueryResponse {data}).unwrap();
                    
                    Response {
                        status: ResponseType::Content,
                        payload
                    }
                },

                SkillRegisterMessage::Notification(m) => {
                    println!("{} wants to notify this data: {:?}", m.skill_id, m.data);

                    Response {
                        status: ResponseType::Content,
                        payload: vec![]
                    }
                }

                SkillRegisterMessage::Close(m) => {
                    println!("{} wants to close", m.skill_id);

                    Response {
                        status: ResponseType::Content,
                        payload: vec![]
                    }
                }
            };

            responder.send(resp).map_err(|_| vap_skill_register::Error::ClosedChannel)?;
        }
    }
}
pub struct MyDataOut{
    out: SkillRegisterOut
}
impl MyDataOut {
    async fn send_request(&mut self, name: String) {
        self.out.activate_skill(name, MsgSkillRequest {
            request_id: 0, // Will be filled by the registry
            client: ClientData {
                system_id: "test-client".into(),
                capabilities: vec![]
            },
            
            request: RequestData {
                type_: RequestDataKind::Intent,
                intent: "hello".into(),
                locale: "en-US".into(),
                slots: vec![]
            }
        }).await.unwrap();
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (reg, stream, out) = SkillRegister::new(conf::PORT).unwrap();
    let (send_name, recv_name) = oneshot::channel();
    let mut m = MyData {name: Some(send_name)};
    let mut m_out = MyDataOut{out};
    let mut request_timer = tokio::time::interval(tokio::time::Duration::from_secs(10));

    let send_requests = async move {
        let name = recv_name.await.unwrap();

        // Wait for client to be fully ready
        tokio::time::sleep(Duration::from_secs(1)).await;
        loop {
            request_timer.tick().await;
            println!("Sending request to: {}", name);
            m_out.send_request(name.clone()).await;
        }
    };

    tokio::select!(
        _= tokio::spawn(reg.run()) => {}
        _= m.on_msg(stream) => {}
        _= send_requests => {}
    );
}
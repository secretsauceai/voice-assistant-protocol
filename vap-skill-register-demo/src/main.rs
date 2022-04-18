use std::collections::HashMap;

use vap_skill_register::{
    SkillRegister, Response, ResponseType, SkillRegisterStream,
    SkillRegisterMessage, 
    structures::{
        MsgConnectResponse, Language, MsgQueryResponse,
        msg_query_response::{QueryData, QueryDataCapability},
    }
};


mod conf {
    pub const PORT: u16 = 5683;
}

struct MyData {

}

impl MyData {
    async fn on_msg(&mut self, mut stream: SkillRegisterStream) -> Result<(), vap_skill_register::Error> {
        loop {
            let (msg, responder) = stream.recv().await?;
            let resp = match msg {
                SkillRegisterMessage::Connect(m) => {
                    println!("{} wants to connect", m.id);
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
                                    if let Some(what) =  x.cap_data.get("what") {
                                        if what == "color" {
                                            let mut res = HashMap::new();
                                            res.insert("color".to_string(), "red".to_string());
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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (reg, stream) = SkillRegister::new("test-skill-register", conf::PORT).unwrap();
    let mut m = MyData {};
    let mut request_timer = tokio::time::interval(tokio::time::Duration::from_secs(10));

    let send_requests = async {
        loop {
            request_timer.tick().await;
            println!("Sending request");
        }
    };

    tokio::select!(
        _= tokio::spawn(reg.run()) => {}
        _= m.on_msg(stream) => {}
        _= send_requests => {}
    );
}
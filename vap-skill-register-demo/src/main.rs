use vap_skill_register::{SkillRegister, Response, ResponseType, SkillRegisterStream, SkillRegisterMessage, structures::{MsgConnectResponse, Language}};
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
                    println!("Someone wants to register this data: {:?}", m.nlu_data);
                    Response {
                        status: ResponseType::Created,
                        payload:  vec![]
                    }
                },
                SkillRegisterMessage::Query(m) => {
                    //println!("Someone wants to query this data: {:?}", m.);
                    Response {
                        status: ResponseType::Content,
                        payload:  vec![]
                    }
                },
                _ => {
                    Response {
                        status: ResponseType::NotImplemented,
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
    tokio::select!(
        _ = tokio::spawn(reg.run()) => {}
        _=  m.on_msg(stream) => {}
    );
}
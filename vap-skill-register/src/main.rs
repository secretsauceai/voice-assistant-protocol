use std::future::Future;
use std::io::Cursor;
use std::net::SocketAddr;


use coap_lite::{RequestType as Method, CoapRequest, CoapResponse};
use coap::Server;
use libmdns::{Responder, Service};
use rmp_serde::from_read;
use serde::de::DeserializeOwned;
use thiserror::Error;
use vap_common_skill::structures::*;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    NameInvalid(std::io::Error),

    #[error("{0}")]
    ZeroconfServiceRegistration(std::io::Error),

    #[error("{0}")]
    ZeroconfPolling(std::io::Error),
}

pub struct SkillRegister {
    name: String,
    port: u16
}

pub enum SkillRegisterMessage {
    Connect(MsgConnect),
    RegisterUtts(MsgRegisterUtts),
    Notification(MsgNotification),
    Query(MsgQuery),
    Close(MsgSkillClose),
}

impl SkillRegister {
    pub fn new(name: &str, port: u16) -> Result<Self, Error> {        
        Ok(SkillRegister {
            name: name.to_string(),
            port
        })
    }

    pub async fn recv(&self) -> Result<(SkillRegisterMessage, Option<CoapResponse>), Error>  {
        let _zeroconf = ZeroconfService::new(&self.name, self.port)?;

        async fn perform(request: CoapRequest<SocketAddr>) -> Option<CoapResponse> {
            fn read_payload<T: DeserializeOwned>(payload: &[u8], r: &Option<CoapResponse>) -> Result<T, Option<CoapResponse>> {
                match from_read(Cursor::new(payload)) {
                    Ok::<T,_>(a) => {
                        Ok(a)
                    }
                    Err(e) => {
                        Err(r.map(|mut r|{
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
                r.map(|mut r| {
                    r.set_status(coap_lite::ResponseType::MethodNotAllowed);
                    r
                })
            }

            fn response_not_implemented(r:Option<CoapResponse>) -> Option<CoapResponse> {
                r.map(|mut r| {
                    r.set_status(coap_lite::ResponseType::NotImplemented);
                    r
                })
            }

            
            match *request.get_method() {
                Method::Get => {
                    match request.get_path().as_str() {
                        "vap/skill_registry/query" => {
                            response_not_implemented(request.response)
                        }

                        _ => response_not_found(request.response)
                    }
                    /*if let Some::<RegisterSkill>(p) = read_payload(&request.message.payload) {
                
                    };*/        
                }

                Method::Post => {                 
                    match request.get_path().as_str() {
                        "vap/skill_registry/connect" => {
                            let p: MsgConnect = read_payload(&request.message.payload, &request.response)?;
                            
                        }

                        "vap/skill_registry/register_utts" => {
                            response_not_implemented(request.response)
                        }

                        "vap/skill_registry/notification" => {
                            response_not_implemented(request.response)
                        }

                        "vap/skill_registry/skill_close" => {
                            response_not_implemented(request.response)
                        }

                        _ => response_not_found(request.response)
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

        let mut server = Server::new(format!("127.0.0.1:{}", self.port)).unwrap();
        server.run(perform).await.unwrap();
        Ok(())   
    }

    pub async fn skills_answerable(&mut self) -> Vec<SkillCanAnswer> {
        // 
        vec![]
    }

    pub async fn activate_skill(&mut self, skill_id: String) {

    }
}

struct ZeroconfService {
    _responder: Responder,
    _service: Service,
}

impl ZeroconfService {
    fn new(name: &str, port:u16) -> Result<ZeroconfService, Error> {
        let _responder = Responder::new().map_err(Error::ZeroconfServiceRegistration)?;
        let _service = _responder.register(
            "_vap-skill-register._udp".to_owned(), 
            name.to_owned(),
            port,
            &["path=/"]
        );
    
        Ok(ZeroconfService{_responder,_service})
    }
}

mod conf {
    pub const PORT: u16 = 5683;
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let reg = SkillRegister::new("test-skill-register", conf::PORT).unwrap();
    reg.on_new_skill(|skill| async move {
        println!("{:?}", skill);
    }).await;

    reg.on_skill_disconnect(|skill| async move {
        println!("{:?}", skill);
    }).await;

}

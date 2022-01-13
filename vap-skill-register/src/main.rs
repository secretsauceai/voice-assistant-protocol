use std::future::Future;
use std::io::Cursor;
use std::net::SocketAddr;

use vap_common_skill::structures::{RegisterSkill, SkillCanAnswer};
use coap_lite::{RequestType as Method, CoapRequest, CoapResponse};
use coap::Server;
use libmdns::{Responder, Service};
use rmp_serde::from_read;
use serde::de::DeserializeOwned;
use thiserror::Error;

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

impl SkillRegister {
    pub fn new(name: &str, port: u16) -> Result<Self, Error> {        
        Ok(SkillRegister {
            name: name.to_string(),
            port
        })
    }

    pub async fn on_new_skill<F, Fut>(&self, cb: F)
    where
        F: Fn(RegisterSkill) -> Fut,
        Fut: Future<Output = ()>
    {
        // TODO: Do this when a skill arrives, and loop this
        // When new skill arrives...
        let data = RegisterSkill{
            id: "org.test.Test".to_string(),
            name: "Skill Test".to_string()
        };

        cb(data).await;
    }

    // TODO: When to unregister a skill with UDP? When we try to connect it
    // and if fails?
    pub async fn on_skill_disconnect<F, Fut>(&self, cb: F)
    where
        F: Fn(RegisterSkill) -> Fut,
        Fut: Future<Output = ()>
    {

    }

    pub async fn serve(&mut self) -> Result<(), Error> {
        let _zeroconf = ZeroconfService::new(&self.name, self.port)?;

        async fn perform(request: CoapRequest<SocketAddr>) -> Option<CoapResponse> {
            fn read_payload<T: DeserializeOwned>(payload: &[u8]) -> Option<T> {
                match from_read(Cursor::new(payload)) {
                    Ok::<T,_>(a) => {
                        Some(a)
                    }
                    Err(e) => {
                        // TODO: Send into logs or others
                        None
                    }
                }
            }

            
            match *request.get_method() {
                Method::Get => {
                    let path = request.get_path();
                    println!("request by get {}", path);

                    if let Some::<RegisterSkill>(p) = read_payload(&request.message.payload) {
                
                    };        
                }

                Method::Post => {
                    // 
                    let path = request.get_path();
                    println!("request by post {}", path);

                    if let Some::<RegisterSkill>(p) = read_payload(&request.message.payload) {

                    };                    
                }

                Method::Put => {
                    // IdemPotency
                    let path = request.get_path();
                    println!("request by put {}", path);

                    if let Some::<RegisterSkill>(p) = read_payload(&request.message.payload) {

                    };                    
                }
                _ => println!("request by other method"),
            };
            
            request.response.map(|mut message| {
                message.message.payload = b"OK".to_vec();
                message
            })
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

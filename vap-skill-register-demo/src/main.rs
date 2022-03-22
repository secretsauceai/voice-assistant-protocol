use vap_skill_register::{SkillRegister, Response, ResponseType, SkillRegisterStream};
mod conf {
    pub const PORT: u16 = 5683;
}

struct MyData {

}

impl MyData {
    async fn on_msg(&mut self, mut stream: SkillRegisterStream) -> Result<(), vap_skill_register::Error> {
        loop {
            let (_msg, responder) = stream.recv().await?;
            responder.send(Response {
                status: ResponseType::NotImplemented,
                payload: vec![]
            }).map_err(|_| vap_skill_register::Error::ClosedChannel)?;
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
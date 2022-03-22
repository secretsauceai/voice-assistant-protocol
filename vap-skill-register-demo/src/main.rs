use vap_skill_register::{SkillRegister, SkillRegisterMessage, Response, ResponseType};

mod conf {
    pub const PORT: u16 = 5683;
}

async fn on_msg(_msg: SkillRegisterMessage) -> Response {
    Response {
        status: ResponseType::NotImplemented,
        payload: vec![]
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (reg, stream) = SkillRegister::new("test-skill-register", conf::PORT).unwrap();
    tokio::select!(
        _= reg.run() => {},
        _= stream.read_incoming(on_msg) => {}
    );
}
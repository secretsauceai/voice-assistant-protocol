use std::io::Cursor;
use std::net::SocketAddr;

use crate::{respond, Response, SkillRegisterMessage};

use coap_lite::{CoapRequest, CoapResponse, ResponseType};
use futures::{channel::{mpsc, oneshot}, SinkExt};
use rmp_serde::from_read;
use serde::de::DeserializeOwned;

pub async fn wait_response<F>(
    receiver: oneshot::Receiver<Response>,
    resp: Option<CoapResponse>,
    cb: F
) -> Option<CoapResponse> where
F: FnOnce(&Response) {
    match receiver.await {
        Ok(resp_data) => {
            cb(&resp_data);
            respond(resp, resp_data.status, resp_data.payload)
        }
        Err(_) => {
            None
        }
    }  
}

pub fn response_not_found(r: Option<CoapResponse>) -> Option<CoapResponse> {
    respond(r, ResponseType::MethodNotAllowed, vec![])
}

pub fn read_payload<T: DeserializeOwned>(payload: &[u8], r: Option<CoapResponse>) -> Result<(T, Option<CoapResponse>), Option<CoapResponse>> {
    match from_read(Cursor::new(payload)) {
        Ok::<T,_>(a) => {
            Ok((a,r))
        }
        Err(e) => {
            Err(r.map(|mut r|{
                println!("{}", &e);
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

pub async fn handle_msg<T: DeserializeOwned, F, F2>(
    request: CoapRequest<SocketAddr>,
    in_send: &mut mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>,
    key_check: F2,
    cb: F,
) -> Option<CoapResponse> where
    F: FnOnce(T) -> SkillRegisterMessage,
    F2: FnOnce(&T) -> bool{

    match read_payload(&request.message.payload, request.response) {
        Ok::<(T,_),_>((p, resp)) => {
            if  key_check(&p){
                let (sender, receiver) = oneshot::channel();
                in_send.send((cb(p), sender)).await.unwrap();
                wait_response(receiver, resp, |_|{}).await
            }
            else {
                respond(resp, ResponseType::BadRequest, vec![])
            }
        }
        Err(r) => {
            r
        }
    }
}
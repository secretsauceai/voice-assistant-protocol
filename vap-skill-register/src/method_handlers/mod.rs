// Handle the incoming CoAP requests

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex as SyncMutex};

use crate::{respond, Notification, NotificationData,  RequestId, RequestResponse, Response, SkillRegisterMessage, SharedPending};
use crate::vars::VAP_VERSION;
use self::io_helpers::*;

use coap_lite::{CoapRequest, CoapResponse, ResponseType};
use futures::future::{join, join_all};
use futures::{channel::{mpsc, oneshot}, SinkExt, lock::Mutex};
use rmp_serde::to_vec_named;
use vap_common_skill::structures::*;

mod io_helpers;

pub async fn on_get(
    request: CoapRequest<SocketAddr>,
    in_send: &mut mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>,
    current_skills: Arc<std::sync::Mutex<HashMap<String, ()>>>
) -> Option<CoapResponse> {
    if request.get_path().starts_with("vap/skillRegistry/skills/") {
        respond(request.response, ResponseType::Content, vec![])
    }

    else {
        match request.get_path().as_str() {
            "vap/skillRegistry/query" => {
                handle_msg(
                    request,
                    in_send,
                    |p: &MsgQuery|current_skills.lock().unwrap().contains_key(&p.skill_id),
                    SkillRegisterMessage::Query
                ).await
            }

            ".well-known/core" => {
                respond(request.response, ResponseType::Content, b"</vap>;rt=\"vap-skill-registry\"".to_vec())
            }

            _ => {
                if request.get_path().starts_with("vap/request/") {
                    // TODO: Make sure only the same skill is asking for it.
                    respond(request.response, ResponseType::Valid, vec![])
                } else {
                    response_not_found(request.response)
                }
            }
        }
    }
}

pub async fn on_post(
    request: CoapRequest<SocketAddr>,
    in_send: &mut mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>,
    self_send: &mut mpsc::Sender<(String, Vec<u8>)>,
    current_skills: &Arc<std::sync::Mutex<HashMap<String, ()>>>,
    pending_can_you: &Arc<Mutex<HashMap<u64, oneshot::Sender<f32>>>>,
    pending_requests: &SharedPending<(Vec<PlainCapability>, oneshot::Sender<RequestResponse>)>
) -> Option<CoapResponse> {
    match request.get_path().as_str() {
        "vap/skillRegistry/connect" => {
            match read_payload(&request.message.payload, request.response) {
                Ok::<(MsgConnect,_),_>((p, resp)) => {
                    if !current_skills.lock().unwrap().contains_key(&p.id) && p.vap_version == VAP_VERSION {
                        let (sender, receiver) = oneshot::channel();
                        let skill_id = p.id.clone();
                        in_send.send((SkillRegisterMessage::Connect(p), sender)).await.unwrap();
                        
                        wait_response(receiver, resp, |r| {
                            // If it is regarded as "OK"
                            if [
                                ResponseType::Created, ResponseType::Deleted,
                                ResponseType::Valid,
                                ResponseType::Changed,
                                ResponseType::Content,
                                ResponseType::Continue
                                ].contains(&r.status) {
                                
                                // We need to register the skill inside the CoAP server
                                self_send.try_send((skill_id.clone(), vec![])).unwrap();
                                current_skills.lock().unwrap().insert(skill_id.clone(),());
                            }
                        }).await
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

        "vap/skillRegistry/registerIntents" => {
            handle_msg(
                request,
                in_send,
                |p: &MsgRegisterIntents|current_skills.lock().unwrap().contains_key(&p.skill_id),
                SkillRegisterMessage::RegisterIntents
            ).await
        }

        "vap/skillRegistry/notification" => {
            
            match read_payload(&request.message.payload, request.response) {
                Ok::<(MsgNotification,_),_>((msg, resp)) => {
                    let mut standalone = vec![];
                    let mut resolutions = vec![];

                    enum RequestResolution {
                        Done(msg_notification_response::Data),
                        InProcess((RequestId, oneshot::Receiver<RequestResponse>))
                    }

                    let skill_id = msg.skill_id;

                    for d in msg.data {
                        match d {
                            msg_notification::Data::CanYouAnswer{request_id, confidence} => {
                                fn can_you_answer_done(response: coap_lite::ResponseType, id: RequestId) -> RequestResolution {
                                    RequestResolution::Done(msg_notification_response::Data::CanYouAnswer {
                                        code: response as u16,
                                        request_id: id
                                    })
                                }

                                let resol= match pending_can_you.lock().await.remove(&request_id) {
                                    Some(pending_sender) => {
                                        pending_sender.send(confidence).unwrap();
                                        can_you_answer_done(coap_lite::ResponseType::Valid, request_id)
                                        
                                    }
                                    None => {
                                        can_you_answer_done(coap_lite::ResponseType::BadRequest, request_id)
                                    }
                                };

                                resolutions.push(resol)
                            }
                            msg_notification::Data::Requested {request_id, capabilities} => {
                                fn requested_done(response: coap_lite::ResponseType, id: RequestId) -> RequestResolution {
                                    RequestResolution::Done(msg_notification_response::Data::Requested {
                                        code: response as u16,
                                        request_id: id
                                    })
                                }

                                let resol = match pending_requests.lock().await.remove(&request_id) {
                                    Some(pending_sender) => {

                                        let (sender, receiver) = oneshot::channel();
                                        pending_sender.send((capabilities.clone(), sender)).unwrap();
                                        RequestResolution::InProcess((request_id, receiver))
                                    }
                                    None => {
                                        requested_done(coap_lite::ResponseType::BadRequest, request_id)
                                    }
                                };

                                resolutions.push(resol)
                            }
                            msg_notification::Data::StandAlone{client_id, capabilities} => {
                                standalone.push(NotificationData {client_id, capabilities});
                            }
                        }
                    }

                    let mut futures = vec![];
                    let mut other_res = vec![];
                    for resolution in resolutions {
                        match resolution {
                            RequestResolution::Done(data) => {
                                other_res.push(data);
                            }
                            RequestResolution::InProcess(receiver) => {
                                futures.push(receiver)                                                
                            }
                        }
                    }
                    let (request_ids, futures) = futures.into_iter().unzip::<_,_,Vec<_>, Vec<_>>();
                    let futs = join_all(futures);                               

                    if !standalone.is_empty() {
                        let send_standalone = async {
                            let (sender, receiver) = oneshot::channel();
                            in_send.send((SkillRegisterMessage::Notification(Notification {
                                skill_id: skill_id.clone(),
                                data: standalone,
                            }), sender)).await.unwrap();
                            wait_response(receiver, resp, |_|{}).await
                        };

                        // TODO: Any result that is not standalone is ignored right now (though it is processed)
                        join(send_standalone, futs).await.0

                    }
                    else {
                        const DEFAULT_RESP: RequestResponse = RequestResponse{code: ResponseType::Content as u16};
                        let res = futs.await.into_iter()
                            .map(|r|r.unwrap_or(DEFAULT_RESP))
                            .zip(request_ids)
                            .map(|(n, request_id)|msg_notification_response::Data::Requested {
                                code: n.code,
                                request_id
                            });
                        other_res.extend(res);
                            
                        resp.map(|mut r| {
                            let payload = to_vec_named(&MsgNotificationResponse {
                                data: other_res
                            }).unwrap();
                            
                            r.set_status(coap_lite::ResponseType::Valid);
                            r.message.payload = payload;
                            r
                        })
                    }
                }
                Err(r) => {
                    r
                }
            }
        }

        _ => response_not_found(request.response)
    }                    
}

pub async fn on_delete(
    request: CoapRequest<SocketAddr>,
    in_send: &mut mpsc::Sender<(SkillRegisterMessage, oneshot::Sender<Response>)>,
    current_skills: Arc<SyncMutex<HashMap<String, ()>>>,
) -> Option<CoapResponse> {
    let path = request.get_path();
    const BASE_SKILLS_PATH: &str = "vap/skillRegistry/skills/";
    if path.starts_with("vap/skillRegistry/skills/") {
        let id = &path[BASE_SKILLS_PATH.len()..];

        match read_payload(&request.message.payload, request.response) {
            Ok::<(MsgSkillClose, _), _>((p, resp)) => {
                if current_skills.lock().unwrap().contains_key(id) {
                    let (sender, receiver) = oneshot::channel();
                    in_send.send((SkillRegisterMessage::Close(p), sender)).await.unwrap();
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
    else {
        response_not_found(request.response)
    }
}
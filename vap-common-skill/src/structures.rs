use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgConnect {
    /// A skill id in the form of org.organization.skill
    pub id: String, 

    /// A human readable name for the skill
    pub name: String ,

    #[serde(rename="vapVersion")]
    pub vap_version: String,

    #[serde(rename="uniqueAuthenticationToken")]
    pub unique_authentication_token: Option<String>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgConnectResponse {
    /// A list of languages currently in use by the voice assistant
    pub langs: Vec<Language>, 

    #[serde(rename="uniqueAuthenticationToken")]
    pub unique_authentication_token: Option<String>
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgRegisterIntents {
    #[serde(rename="skillId")]
    pub skill_id: String,

    #[serde(rename="nluData")]
    pub nlu_data: Vec<msg_register_intents::NluData>
}

pub mod msg_register_intents {
    use super::Language;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct NluData {
        pub language: Language,
        pub intents: Vec<NluDataIntent>,
        pub entities: Vec<NluDataEntity>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct NluDataIntent {
        pub name: String,
        pub utterances: Vec<NluDataIntentUtterance>,
        pub slots: Vec<NluDataSlot>
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct NluDataIntentUtterance {
        pub text: String
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct NluDataSlot {
        pub name: String,
        pub entity: String
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct NluDataEntity {
        pub strict: bool,
        pub data: Vec<NluDataEntityData>
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct NluDataEntityData {
        pub value: String,
        pub synonyms: Vec<String>
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Language { // Better this or a single string?
    /// The country code of the language
    pub country: Option<String>, 

    /// The language code
    pub language: String, 

    /// The extra code for the language
    pub extra: Option<String>, // is this necessary?
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgRegisterIntentsResponse {
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgSkillRequest {
    pub request_id: u64,
    pub client: msg_skill_request::ClientData,
    pub request: msg_skill_request::RequestData,
}

pub mod msg_skill_request {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ClientData {
        #[serde(rename="systemId")]
        pub system_id: String,
        pub capabilities: Vec<ClientDataCapability>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ClientDataCapability {
        pub name: String,
        pub version: u16,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub enum RequestDataKind {
        #[serde(rename="intent")]
        Intent,

        #[serde(rename="event")]
        Event,
        
        #[serde(rename="canAnswer")]
        CanAnswer
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct RequestData {
        #[serde(rename="type")]
        pub type_: RequestDataKind,
        pub intent: String,
        pub locale: String,
        pub slots: Vec<RequestSlot>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct RequestSlot {
        pub name: String,
        pub value: Option<String>,
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgNotification {
    #[serde(rename="skillId")]
    pub skill_id: String,

    pub data: Vec<msg_notification::Data>
}

pub mod msg_notification {
    use serde::{Deserialize, Serialize};
    
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(tag="type")]
    pub enum Data {
        #[serde(rename="requested")]
        Requested {
            #[serde(rename="requestId")]
            request_id: u64,

            capabilities: Vec<super::PlainCapability>,
        },

        #[serde(rename="standalone")]
        StandAlone {
            #[serde(rename="clientId")]
            client_id: String,

            capabilities: Vec<super::PlainCapability>
        },
        
        #[serde(rename="canYouAnswer")]
        CanYouAnswer {
            #[serde(rename="requestId")]
            request_id: u64,
            
            confidence: f32
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgNotificationResponse {
    pub data: Vec<msg_notification_response::Data>
}

pub mod msg_notification_response {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(tag="type")]
    pub enum Data {
        #[serde(rename="requested")]
        Requested {
            #[serde(rename="requestId")]
            request_id: u64,
            code: u16,
        },

        #[serde(rename="standalone")]
        StandAlone {
            #[serde(rename="clientId")]
            client_id: String,
            code: u16,
        },
        
        #[serde(rename="canYouAnswer")]
        CanYouAnswer {
            #[serde(rename="requestId")]
            request_id: u64,
            code: u16,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgQuery {
    #[serde(rename="skillId")]
    pub skill_id: String,
    pub data: Vec<msg_query::QueryData>
}

mod msg_query {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct QueryData {
        #[serde(rename="clientId")]
        pub client_id: String,
        pub capabilities: Vec<super::PlainCapability>
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgQueryResponse {
    pub data: Vec<msg_query_response::QueryData>
}

pub mod msg_query_response {
    use std::collections::HashMap;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct QueryData {
        #[serde(rename="clientId")]
        pub client_id : String,
        pub capabilities: Vec<QueryDataCapability>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct QueryDataCapability {
        pub name: String,
        pub code: u16,

        #[serde(flatten)]
        pub data: HashMap<String, String>,
    }

}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgSkillClose {
    #[serde(rename="skillId")]
    pub skill_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlainCapability {
    pub name: String,

    #[serde(flatten)]
    pub cap_data: HashMap<String, String> // TODO: Make this some kind of value thing
}
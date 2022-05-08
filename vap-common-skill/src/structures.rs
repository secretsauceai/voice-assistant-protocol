use std::{collections::HashMap, hash::Hash, fmt::Display};

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
        pub name: String,
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
    use serde::{Deserialize, Serialize};

    use super::AssociativeMap;

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
        pub data: AssociativeMap,
    }

}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgSkillClose {
    #[serde(rename="skillId")]
    pub skill_id: String,
}

/// A structure describing Capability data
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlainCapability {
    pub name: String,

    #[serde(flatten)]
    pub cap_data: AssociativeMap
}

pub type AssociativeMap = HashMap<Value, Value>;

/// Used as variant for the capabilities data. Represents all types in MsgPack
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)] 
pub enum Value {
    Nil,
    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),
    Binary(Vec<u8>),
    Array(Vec<Value>),
    Map(HashMap<Value, Value>),
    // Timestamp // TODO! Finish this type
}


/// PartialEq implementation. For floating point instead of direct equality, we
/// we make sure they are close enough (because of how computers treat decimals).
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::I8(l0), Self::I8(r0)) => l0 == r0,
            (Self::U8(l0), Self::U8(r0)) => l0 == r0,
            (Self::I16(l0), Self::I16(r0)) => l0 == r0,
            (Self::U16(l0), Self::U16(r0)) => l0 == r0,
            (Self::I32(l0), Self::I32(r0)) => l0 == r0,
            (Self::U32(l0), Self::U32(r0)) => l0 == r0,
            (Self::I64(l0), Self::I64(r0)) => l0 == r0,
            (Self::U64(l0), Self::U64(r0)) => l0 == r0,
            (Self::F32(l0), Self::F32(r0)) => (l0 - r0) < std::f32::EPSILON, 
            (Self::F64(l0), Self::F64(r0)) => (l0 - r0) < std::f64::EPSILON,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Binary(l0), Self::Binary(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::Map(l0), Self::Map(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl Eq for Value {
}

impl Display for Value {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn vec_to_string<D: Display>(v: &[D]) -> String {
            v.iter().map(|v|v.to_string()).collect::<Vec<String>>().join(", ")
        }

        fn map_to_string<D1: Display, D2: Display>(m: &HashMap<D1, D2>) -> String {
            m.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<String>>().join(", ")
        }

        match self {
            Value::Nil => fmt.write_str("Nil"),
            Value::Bool(b) => fmt.write_str(&b.to_string()),
            Value::I8(i) => fmt.write_str(&i.to_string()),
            Value::U8(u) => fmt.write_str(&u.to_string()),
            Value::I16(i) => fmt.write_str(&i.to_string()),
            Value::U16(u) => fmt.write_str(&u.to_string()),
            Value::I32(i) => fmt.write_str(&i.to_string()),
            Value::U32(u) => fmt.write_str(&u.to_string()),
            Value::I64(i) => fmt.write_str(&i.to_string()),
            Value::U64(u) => fmt.write_str(&u.to_string()),
            Value::F32(f) => fmt.write_str(&f.to_string()),
            Value::F64(f) => fmt.write_str(&f.to_string()),
            Value::String(str) => fmt.write_str(str),
            Value::Binary(b) => fmt.write_str(&vec_to_string(b)),
            Value::Array(a) => fmt.write_str(&vec_to_string(a)),
            Value::Map(m) => fmt.write_str(&map_to_string(m)),
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}
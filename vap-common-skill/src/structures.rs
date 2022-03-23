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
pub struct MsgSkillCanAnswer {
    // - pub request
    // - pub slot
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgSkillCanAnswerResponse {
    // - pub request
    // - pub slot
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgSkillRequest {
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
    pub struct RequestData {
        #[serde(rename="type")]
        pub type_: String,
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
pub struct MsgSkillRequestResponse {
    pub capabilities: Vec<msg_skill_request_response::Capability>,
}

mod msg_skill_request_response {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Capability {
        pub name: String,
        pub data: String, // TODO: This should be a mapping
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgNotification {
    //
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgQuery {
    //
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgSkillClose {
    #[serde(rename="skillId")]
    pub skill_id: String,
}
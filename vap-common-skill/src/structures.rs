use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgConnect {
    /// A skill id in the form of org.organization.skill
    pub id: String, 

    /// A human readable name for the skill
    pub name: String ,

    
    pub vap_version: String,

    pub unique_authentication_token: Option<String>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgConnectResponse {
    /// A list of languages currently in use by the voice assistant
    pub langs: Vec<Language>, 

    pub unique_authentcation_token: Option<String>,

    pub connection_authentication_token: Option<String>
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgRegisterUtts {
    // pub utterances: One set per language
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
pub struct ProcessingResult {
    
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillCanAnswer {
    // - pub request
    // - pub slot
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillAnswer {
    // pub isfinal
    // pub capabilities
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
    pub skill_id: String,
    pub connection_authorization_token: String
}
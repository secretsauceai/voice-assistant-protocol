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
pub struct MsgRegisterUtts {
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
pub struct MsgRegisterUttsResponse {
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
pub struct MsgSkillAnswer {
    // pub isfinal
    // pub capabilities
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MsgSkillAnswerResponse {
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
    #[serde(rename="skillId")]
    pub skill_id: String,
}
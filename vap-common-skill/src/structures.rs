use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterSkill {
    /// A skill id in the form of org.organization.skill
    pub id: String, 

    /// A human readable name for the skill
    pub name: String 
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InitialSkillConfig { // TODO: Add the possibility of some error
    /// A list of languages currently in use by the voice assistant
    pub langs: Vec<Language>, 

    /** A unique ID for the skill issued by the skill register
     *  this is made so that skill can ignore fraudulent
     * messages
     */
    pub skill_uuid: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InitialUtterances {
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
pub struct SkillNotify {
    //
}
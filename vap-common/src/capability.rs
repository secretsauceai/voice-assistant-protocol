pub enum Capability {
    Sound,
    Text, //should this be a separate capability?
    Image,
    WakeWordSync,
    WakeWordAudio,
    Log,
    DynamicNLU,
}

pub struct CapabilityCode(u8);

impl From<&str> for Capability {
    fn from(capability: &str) -> Self {
        match capability.to_lowercase().as_str() {
            "sound" => Self::Sound,
            "text" => Self::Text,
            "image" => Self::Image,
            "wakewordsync" => Self::WakeWordSync,
            "wakewordaudio" => Self::WakeWordAudio,
            "log" => Self::Log,
            "dynamicnlu" => Self::DynamicNLU,
            _ => panic!("Unknown capability: {}", capability),
        }
    }
}

impl From<CapabilityCode> for Capability {
    fn from(capability: CapabilityCode) -> Self {
        match capability.0 {
            0 => Self::Sound,
            1 => Self::Text,
            2 => Self::Image,
            3 => Self::WakeWordSync,
            4 => Self::WakeWordAudio,
            5 => Self::Log,
            6 => Self::DynamicNLU,
            _ => panic!("Unknown capability: {}", capability.0),
        }
    }
}

impl From<Capability> for CapabilityCode {
    fn from(capability: Capability) -> Self {
        match capability {
            Capability::Sound => Self(0),
            Capability::Text => Self(1),
            Capability::Image => Self(2),
            Capability::WakeWordSync => Self(3),
            Capability::WakeWordAudio => Self(4),
            Capability::Log => Self(5),
            Capability::DynamicNLU => Self(6),
        }
    }
}

impl CapabilityCode {
    pub fn to_u8(&self) -> u8 {
        self.0
    }
}
// pub struct Capabilities {
//     capabilities: &'static [CapabilityCode],
// }

// impl Default for Capabilities {
//     fn default() -> Self {
//         Self { capabilities: &[] }
//     }
// }

// impl Capabilities {
//     pub fn new(capabilities: &[Capabilities]) -> Self {
//         caps = [capabilities.len()]
//     }
// }

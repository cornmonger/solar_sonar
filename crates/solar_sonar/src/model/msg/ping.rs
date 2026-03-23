use crate::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq,
    serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode
)]
pub enum PingKind {
    Combat,
    Danger,
    Intel,
    Message,
}

impl PingKind {
    pub fn try_from_input(s: &str) -> SolarResult<Self> {
        match s.to_lowercase().as_str() {
            "combat" => Ok(Self::Combat),
            "danger" => Ok(Self::Danger),
            "intel" => Ok(Self::Intel),
            "message" => Ok(Self::Message),
            _ => SolarError::err_enum(ErrNoun::PingKind, s), 
        }
    }
}

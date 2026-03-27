use crate::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq,
    serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode
)]
pub enum AnalysisKind {
    Callout,
    Combat,
    Intel,
    Message,
}

impl AnalysisKind {
    pub fn try_from_input(s: &str) -> SolarResult<Self> {
        match s.to_lowercase().as_str() {
            "callout" => Ok(Self::Callout),
            "combat" => Ok(Self::Combat),
            "intel" => Ok(Self::Intel),
            "message" => Ok(Self::Message),
            _ => SolarError::err_enum(ErrNoun::PingKind, s), 
        }
    }
}

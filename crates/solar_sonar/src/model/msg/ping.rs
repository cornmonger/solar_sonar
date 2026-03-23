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

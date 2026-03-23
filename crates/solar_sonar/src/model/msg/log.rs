use crate::*;


pub type ChannelId = u64;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash,
    serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
#[repr(u8)]
pub enum LogKind {
    Game,
    Chat(ChatLogKind),
}

impl LogKind {
    pub fn try_from_enum(s: String) -> SolarResult<Self> {
        if let Ok(chat_kind) = ChatLogKind::try_from_enum(&s) {
            Ok(Self::Chat(chat_kind))
        } else if s.to_lowercase() == "game" {
            Ok(Self::Game)
        } else {
            Err(SolarError::msg("Invalid log kind"))
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash,
    serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
#[repr(u8)]
pub enum ChatLogKind {
    Alliance,
    Corporation,
    Fleet,
    Group,
    Local,
    Private,
}

impl ChatLogKind {
    pub fn try_from_enum(s: &str) -> SolarResult<Self> {
        match s.to_lowercase().as_str() {
            "alliance" => Ok(Self::Alliance),
            "corp" => Ok(Self::Corporation),
            "local" => Ok(Self::Local),
            "fleet" => Ok(Self::Fleet),
            _ => Err(SolarError::msg("Invalid chat log kind")),
        }
    }
    
    pub fn from_log_name(name: &str) -> Self {
        match name {
            "Alliance" => Self::Alliance,
            "Corp" => Self::Corporation,
            "Local" => Self::Local,
            "Fleet" => Self::Fleet,
            "Private" => Self::Private,
            _ => Self::Group,
        }
    }
}

use crate::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash,
    serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub enum CharacterLog {
    Game { character_id: CharacterId },
    Local { character_id: CharacterId },
    Alliance { character_id: CharacterId },
    Corporation { character_id: CharacterId },
    Fleet { character_id: CharacterId },
    Group { character_id: CharacterId, channel_id: IndexId },
}

impl CharacterLog {
    pub fn from_kind(kind: LogKind, character_id: CharacterId, channel_id: Option<IndexId>) -> Self {
        match kind {
            LogKind::Game => Self::Game { character_id },
            LogKind::Local => Self::Local { character_id },
            LogKind::Alliance => Self::Alliance { character_id },
            LogKind::Corporation => Self::Corporation { character_id },
            LogKind::Fleet => Self::Fleet { character_id },
            LogKind::Group => Self::Group { character_id, channel_id: channel_id.expect("channel id") },
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash,
    serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
#[repr(u8)]
pub enum LogKind {
    Game,
    Local,
    Alliance,
    Corporation,
    Fleet,
    Group,
}

impl LogKind {
    pub fn try_from_input(s: &str) -> SolarResult<Self> {
        match s.to_lowercase().as_str() {
            "game" => Ok(Self::Game),
            "local" => Ok(Self::Local),
            "alliance" => Ok(Self::Alliance),
            "corp" => Ok(Self::Corporation),
            "fleet" => Ok(Self::Fleet),
            "group" => Ok(Self::Group),
            _ => SolarError::err_enum(ErrNoun::LogKind, s)
        }
    }
    
    pub fn from_log_name(name: &str) -> Self {
        match name {
            "Game" => Self::Game,
            "Local" => Self::Local,
            "Alliance" => Self::Alliance,
            "Corp" => Self::Corporation,
            "Fleet" => Self::Fleet,
            "Private" => panic!("Unsupported log kind: Private"),
            _ => Self::Group,
        }
    }
}

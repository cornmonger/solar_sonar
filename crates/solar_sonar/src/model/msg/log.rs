use crate::*;

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
    
    pub fn from_log_file(file: &ChatLogFile) -> Self {
        match file.name() {
            LogName::Game => Self::Game,
            LogName::Chat(name) => match *name {
                "Game" => Self::Game,
                "Local" => Self::Local,
                "Alliance" => Self::Alliance,
                "Corp" => Self::Corporation,
                "Fleet" => Self::Fleet,
                "Private" => panic!("Unsupported log kind: Private"),
                _ => Self::Group,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LogNameString {
    Game,
    Local,
    Alliance,
    Corporation,
    Fleet,
    Group(String),
}

impl LogNameString {
    pub(crate) fn from_log_file(file: ChatLogFile) -> SolarResult<Self> {
        let kind = LogKind::from_log_file(&file);
        let log_name = file.name();
        
        match kind {
            LogKind::Game => Ok(Self::Game),
            LogKind::Local => Ok(Self::Local),
            LogKind::Alliance => Ok(Self::Alliance),
            LogKind::Corporation => Ok(Self::Corporation),
            LogKind::Fleet => Ok(Self::Fleet),
            LogKind::Group => match log_name { 
                LogName::Chat(name) => Ok(Self::Group(name.to_string())),
                _ => Err(SolarError::msg(format!("Grup chat name not found for log file: {}", file.path().to_string_lossy())))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IndexedLogName {
    Game,
    Local,
    Alliance,
    Corporation,
    Fleet,
    Group(IndexId),
}

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
    pub fn character_id(&self) -> CharacterId {
        match self {
            Self::Game { character_id } 
            | Self::Local { character_id }
            | Self::Alliance { character_id }
            | Self::Corporation { character_id }
            | Self::Fleet { character_id }
            | Self::Group { character_id, .. } => *character_id
        }
    }
    
    pub fn channel_name_id(&self) -> Option<IndexId> {
        match self {
            Self::Group { channel_id,.. } => Some(*channel_id),
            _ => None,
        }
    }
    
    pub fn display_str<'a>(&self, index: &'a Index) -> &'a str {
        match self {
            Self::Game {..} => "Game",
            Self::Local {..} => "Local",
            Self::Alliance {..} => "Alliance",
            Self::Corporation {..} => "Corp",
            Self::Fleet {..} => "Fleet",
            Self::Group { channel_id, .. } => &index.chat_channels().get(*channel_id).expect("channel indexed").name,
        }
    }
    
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
    
    pub fn to_kind(&self) -> LogKind {
        match self {
            Self::Game {..} => LogKind::Game,
            Self::Local {..} => LogKind::Local,
            Self::Alliance {..} => LogKind::Alliance,
            Self::Corporation {..} => LogKind::Corporation,
            Self::Fleet {..} => LogKind::Fleet,
            Self::Group {..} => LogKind::Group,
        }
    }
}

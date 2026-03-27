//! We describe logs in different ways:
//! - [LogKind] a fieldless enum that describes only the type of log.
//! - [NamedLog] extends log kind to include a String channel name for Group chat logs 
//! - [IndexedLog] extends log kind to include an indexed ID of a Group channel name instead of a String
//! - [CharacterLog] extends an indexed log to include a specific character id
//! 
//! [NamedLog] is the only one that is not Copy. It is primarly as an intermediary
//! between user input and actual data.
//! 
//! We also model the parent directory of a log path via [LogDirKind], which
//! allows us to determine the log kind without opening a file.
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
    
    pub(crate) fn from_log_file(file: &LogFile) -> Self {
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
    
    pub fn pronounce(&self) -> &'static str {
        match self {
            Self::Game {..} => "Game",
            Self::Local {..} => "Local",
            Self::Alliance {..} => "Alliance",
            Self::Corporation {..} => "Corp",
            Self::Fleet {..} => "Fleet",
            Self::Group {..} => "Chat",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NamedLog {
    Game,
    Local,
    Alliance,
    Corporation,
    Fleet,
    Group(String),
}

impl NamedLog {
    pub(crate) fn from_log_file(file: LogFile) -> SolarResult<Self> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexedLog {
    Alliance,
    Corporation,
    Fleet,
    Game,
    Local,
    Group(IndexId),
}

impl IndexedLog {
    pub fn try_from_kind(kind: LogKind, name: Option<&str>, channel_name_index: &ChannelNameIndex) -> SolarResult<Self> {
        match kind {
            LogKind::Alliance => Ok(Self::Alliance),
            LogKind::Corporation => Ok(Self::Corporation),
            LogKind::Fleet => Ok(Self::Fleet),
            LogKind::Game => Ok(Self::Game),
            LogKind::Local => Ok(Self::Local),
            LogKind::Group => {
                let Some(name) = name else {
                    return SolarError::err_msg("Log kind expects Group, but channel name was not provided");
                };
                
                let id = channel_name_index.find(name)?.id();
                Ok(Self::Group(id))
            }
        }
    }
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
    
    pub fn is_chat(&self) -> bool {
        match self {
            Self::Game {..} => false,
            _ => true,
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
    
    pub(crate) fn from_indexed(indexed: IndexedLog, character_id: CharacterId) -> Self {
        match indexed {
            IndexedLog::Game => Self::Game { character_id },
            IndexedLog::Local => Self::Local { character_id },
            IndexedLog::Alliance => Self::Alliance { character_id },
            IndexedLog::Corporation => Self::Corporation { character_id },
            IndexedLog::Fleet => Self::Fleet { character_id },
            IndexedLog::Group(channel_id) => Self::Group { character_id, channel_id },
        }
    }
    
    pub(crate) fn to_indexed(&self) -> IndexedLog {
        match self {
            Self::Game {..} => IndexedLog::Game,
            Self::Local {..} => IndexedLog::Local,
            Self::Alliance {..} => IndexedLog::Alliance,
            Self::Corporation {..} => IndexedLog::Corporation,
            Self::Fleet {..} => IndexedLog::Fleet,
            Self::Group {channel_id,..} => IndexedLog::Group(*channel_id),
        }
    }
}

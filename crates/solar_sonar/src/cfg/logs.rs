use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LogsCfg {
    #[serde(rename = "log")]
    pub logs: Vec<LogCfg>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LogCfg {
    pub kind: String,
    pub modes: Vec<String>,
    pub characters: Vec<String>,
    pub pings: Vec<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogConfig {
    pub kind: LogKind,
    pub modes: Vec<IndexId>,
    pub characters: Vec<CharacterId>,
    pub pings: Vec<PingKind>,
    /// applicable to group chat channels only
    pub name: Option<IndexId>,
}

impl CfgToml for LogsCfg {
    const TOML_FILENAME: &'static str = "logs.toml";
    const DEFAULT_TOML: &'static str = include_str!("../../assets/config/default/logs.toml");
}

impl LogConfig {
    pub(crate) fn try_from_cfg(cfg: LogCfg, mode_index: &ModeIndex, character_index: &CharacterIndex, channel_index: &ChannelNameIndex) -> SolarResult<Self> {
        let kind = LogKind::try_from_input(&cfg.kind)?;
        let modes = cfg.modes.into_iter()
            .map(|m| mode_index.find(&m).map(|idx| idx.id()))
            .collect::<SolarResult<Vec<_>>>()?;
        let characters = cfg.characters.into_iter()
            .map(|c| character_index.find(&c).map(|idx| idx.character_id()))
            .collect::<SolarResult<Vec<_>>>()?;
        let pings = cfg.pings.into_iter()
            .map(|p| PingKind::try_from_input(&p))
            .collect::<SolarResult<Vec<_>>>()?;
        
        let name = match(kind, cfg.name) {
            (LogKind::Group, Some(name)) => Ok(Some(channel_index.find(&name).expect("indexed").id())),
            (LogKind::Group, None) => SolarError::err_msg(format!("Name is required for group chat logs")),
            _ => Ok(None),
        }?;
        
        Ok(Self { kind, modes, characters, pings, name })
    }
}

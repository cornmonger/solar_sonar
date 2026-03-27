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
    pub indexed: IndexedLog,
    pub modes: Vec<IndexId>,
    pub characters: Vec<CharacterId>,
    pub analysis: Vec<AnalysisKind>,
}

impl CfgToml for LogsCfg {
    const TOML_FILENAME: &'static str = "logs.toml";
    const DEFAULT_TOML: &'static str = include_str!("../../assets/config/default/logs.toml");
}

impl LogConfig {
    pub(crate) fn try_from_cfg(cfg: LogCfg, mode_index: &ModeIndex, character_index: &CharacterIndex, channel_index: &ChannelNameIndex) -> SolarResult<Self> {
        let kind = LogKind::try_from_input(&cfg.kind)?;
        let indexed = IndexedLog::try_from_kind(kind, cfg.name.as_deref(), channel_index)?;
        
        let modes = cfg.modes.into_iter()
            .map(|m| mode_index.find(&m).map(|idx| idx.id()))
            .collect::<SolarResult<Vec<_>>>()?;
        let characters = cfg.characters.into_iter()
            .map(|c| character_index.find(&c).map(|idx| idx.character_id()))
            .collect::<SolarResult<Vec<_>>>()?;
        let pings = cfg.pings.into_iter()
            .map(|p| AnalysisKind::try_from_input(&p))
            .collect::<SolarResult<Vec<_>>>()?;
        
        Ok(Self { indexed, modes, characters, analysis: pings })
    }
}

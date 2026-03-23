use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LogsCfg {
    #[serde(rename = "log")]
    pub logs: LogCfg,
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
}

impl LogConfig {
    pub(crate) fn try_from_cfg(cfg: LogCfg, all_modes: &HashMap<String, IndexId>) -> SolarResult<Self> {
        let kind = LogKind::try_from_enum(cfg.kind)?;
        let modes = cfg.modes.into_iter()
            .map(|m| all_modes.get(&m).ok_or_else(|| SolarError::msg("Invalid mode")))
            .collect::<SolarResult<Vec<_>>>()?;
        
        todo!()
    }
}

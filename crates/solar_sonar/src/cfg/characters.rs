use crate::*;

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CharacterConfig {
    pub alias: String,
    pub id: CharacterId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CharacterCfg {
    pub alias: String,
    pub id: CharacterId,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CharactersCfg {
    pub character: Vec<CharacterCfg>,
}

impl CfgToml for CharactersCfg {
    const TOML_FILENAME: &'static str = "characters.toml";
    const DEFAULT_TOML: &'static str = include_str!("../../assets/config/default/characters.toml");
}

impl CharacterConfig {
    pub(crate) fn try_from_cfg(cfg: CharacterCfg) -> Self {
        Self {
            alias: cfg.alias,
            id: cfg.id,
            name: cfg.name,
        }
    }
}

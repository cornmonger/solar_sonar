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
    #[serde(rename = "character")]
    pub characters: Vec<CharacterCfg>,
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

impl IndexKey for CharacterConfig {
    fn index_key(&self) -> &str { &self.alias }
}

#[derive(Debug)]
pub struct CharacterCfgConst {
    pub alias: &'static str,
    pub id: CharacterId,
    pub name: &'static str,
}

impl From<&CharacterCfgConst> for CharacterCfg {
    fn from(v: &CharacterCfgConst) -> Self {
        Self {
            alias: v.alias.to_string(),
            id: v.id,
            name: v.name.to_string(),
       }
    }
}


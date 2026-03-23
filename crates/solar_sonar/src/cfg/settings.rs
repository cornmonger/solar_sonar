use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SettingsCfg {
    pub logs_dir: PathBuf,
    pub modes: Vec<String>,
}

impl CfgToml for SettingsCfg {
    const TOML_FILENAME: &'static str = "settings.toml";
    const DEFAULT_TOML: &'static str = include_str!("../../assets/config/default/settings.toml");
}

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

#[derive(Debug)]
pub struct SettingsCfgConst {
    pub logs_dir: &'static str,
    pub modes: &'static [&'static str],
}

impl From<&SettingsCfgConst> for SettingsCfg {
    fn from(v: &SettingsCfgConst) -> Self {
        Self {
            logs_dir: PathBuf::from(v.logs_dir),
            modes: v.modes.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        }
    }
}
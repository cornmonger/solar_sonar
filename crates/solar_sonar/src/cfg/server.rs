use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ServerCfg {
    pub default: Option<String>,
    pub serve: Vec<ServeCfg>,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerProfileConfig {
    pub name: String,
    pub tls: TlsConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ServeCfg {
    pub name: String,
    #[serde(flatten)]
    pub tls: TlsCfg,
}

impl CfgToml for ServerCfg {
    const TOML_FILENAME: &'static str = "server.toml";
    const DEFAULT_TOML: &'static str = include_str!("../../assets/config/default/server.toml");
}

impl ServerProfileConfig {
    pub(crate) fn try_from_cfg(cfg: ServeCfg) -> SolarResult<Self> {
        let tls = TlsConfig::try_from_cfg(cfg.tls)?;
        
        Ok(Self {
            name: cfg.name,
            tls,
        })
    }
}

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClientCfg {
    pub default: Option<String>,
    // optional list of connections
    #[serde(default)]
    pub connect: Vec<ConnectCfg>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConnectCfg {
    pub name: String,
    #[serde(flatten)]
    pub tls: TlsCfg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientProfileConfig {
    pub name: String,
    pub tls: TlsConfig,
}

impl CfgToml for ClientCfg {
    const TOML_FILENAME: &'static str = "client.toml";
    const DEFAULT_TOML: &'static str = include_str!("../../assets/config/default/client.toml");
}

impl ClientProfileConfig {
    pub(crate) fn try_from_cfg(cfg: ConnectCfg) -> SolarResult<Self> {
        let tls = TlsConfig::try_from_cfg(cfg.tls)?;
        
        Ok(Self {
            name: cfg.name,
            tls,
        })
    }
}

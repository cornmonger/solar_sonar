use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClientCfg {
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

#[derive(Debug)]
pub struct ClientCfgConst {
    pub connect: &'static [ConnectCfgConst],
}

#[derive(Debug)]
pub struct ConnectCfgConst {
    pub name: &'static str,
    pub tls: TlsCfgConst,
}

impl From<&ClientCfgConst> for ClientCfg {
    fn from(v: &ClientCfgConst) -> Self {
        Self {
            connect: v.connect.iter()
                .map(|c| c.into())
                .collect(),
        }
    }
}

impl From<&ConnectCfgConst> for ConnectCfg {
    fn from(v: &ConnectCfgConst) -> ConnectCfg {
        Self {
            name: v.name.to_string(),
            tls: (&v.tls).into(),
        }
    }
}

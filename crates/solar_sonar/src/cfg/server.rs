use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ServerCfg {
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

#[derive(Debug)]
pub struct ServerCfgConst {
    pub serve: &'static [ServeCfgConst],
}

#[derive(Debug)]
pub struct ServeCfgConst {
    pub name: &'static str,
    pub tls: TlsCfgConst,
}

impl From<&ServeCfgConst> for ServeCfg {
    fn from(v: &ServeCfgConst) -> Self {
        Self {
            name: v.name.to_string(),
            tls: (&v.tls).into(),
        }
    }
}

impl From<&ServerCfgConst> for ServerCfg {
    fn from(v: &ServerCfgConst) -> Self {
        Self {
            serve: v.serve.iter()
                .map(|c| c.into())
                .collect(),
        }
    }
}


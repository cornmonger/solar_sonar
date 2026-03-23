use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TlsCfg {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsConfig {
    pub ip: IpAddr,
    pub port: u16,
}

impl TlsConfig {
    pub fn try_from_cfg(cfg: TlsCfg) -> SolarResult<Self> {
        let ip =  cfg.ip.parse()
            .map_err(|_| SolarError::msg(format!("Invalid TLS server IP: {}", cfg.ip)))?;
        let port = cfg.port;
        
        Ok(TlsConfig { ip, port })
    }
}

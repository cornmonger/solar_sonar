use crate::*;

pub(crate) const CONFIG_DIR: &'static str = ".config/solar_sonar";
pub(crate) const CERTS_DIR: &'static str = "certs";
const DEFAULT_CHARACTERS_TOML: &'static str = include_str!("../assets/config/default/characters.toml");
const DEFAULT_SETTINGS_TOML: &'static str = include_str!("../assets/config/default/settings.toml");
const DEFAULT_SERVER_TOML: &'static str = include_str!("../assets/config/default/server.toml");
const DEFAULT_CLIENT_TOML: &'static str = include_str!("../assets/config/default/client.toml");

// Fully processing configuration. Consumed internally. Available publically
// primarily for inspection.
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub logs_dir: PathBuf,
    pub characters: Vec<CharacterConfig>,
    pub server_profiles: Vec<ServerProfileConfig>,
    pub client_profiles: Vec<ClientProfileConfig>,
    pub default_client_profile: Option<String>,
    pub default_server_profile: Option<String>,
}

/// Public-facing configuration. Converted into [Config] at init.
/// Call [Cfg::read], [Cfg::read_dir], or [Cfg::build] to prepare.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Cfg {
    pub characters: Vec<CharacterCfg>,
    pub settings: SettingsCfg,
    pub server: ServerCfg,
    pub client: ClientCfg,
}

/// The result of building a [Cfg].
#[derive(Debug, PartialEq, Eq)]
pub struct CfgParam {
    pub config: Config,
    pub index: Index,
}

impl Cfg {
    pub fn read() -> SolarResult<CfgParam> {
        let config_dir = SolarSonar::get().config_dir();
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| SolarError::mkdir(e, &config_dir))?;
        }

        Self::read_dir(&config_dir)
    }

    pub fn read_dir(config_dir: &Path) -> SolarResult<CfgParam> {
        let config_dir = match config_dir.is_absolute() {
            true => Cow::Borrowed(config_dir),
            false => expand_path(config_dir)?,
        };
        
        let settings = SettingsCfg::read(&config_dir)?;

        let characters = CharactersCfg::read(&config_dir)?
            .character.into_iter()
            .collect::<Vec<_>>();
        
        let server = ServerCfg::read(&config_dir)?;
        let client = ClientCfg::read(&config_dir)?;
        
        Cfg { characters, settings, server, client }.build()
    }

    pub fn build(self) -> SolarResult<CfgParam> {
        //todo: pull from logs.toml
        let channel_names = self.characters.iter()
            .map(|chr| &chr.intel_channels)
            .flatten()
            .map(String::clone)
            .collect::<Vec<_>>();
        
        let chat_channels = ChatChannels::try_new(channel_names)?;
        let characters = self.characters.into_iter()
            .map(|chr| CharacterConfig::from_cfg(chr, &chat_channels))
            .collect::<Vec<_>>();

        let logs_dir = shellexpand::path::full(&self.settings.logs_dir)
            .map_err(|_| SolarError::msg(format!("Environment expansion failed for settings.log_dir: {}", self.settings.logs_dir.to_string_lossy())))?;
        
        let logs_dir = match logs_dir {
            Cow::Borrowed(p) => p.into(),
            Cow::Owned(p) => p,
        };
        
        let modes = ModeIndex::try_new(self.settings.modes)?;
        
        let server_profiles = self.server.serve.into_iter()
            .map(|cfg| ServerProfileConfig::try_from_cfg(cfg))
            .collect::<SolarResult<Vec<_>>>()?;
        let client_profiles = self.client.connect.into_iter()
            .map(|cfg| ClientProfileConfig::try_from_cfg(cfg))
            .collect::<SolarResult<Vec<_>>>()?;
        
        let default_server_profile = self.server.default;
        let default_client_profile = self.client.default;
        
        let config = Config {
            logs_dir,
            characters,
            server_profiles,
            client_profiles,
            default_server_profile,
            default_client_profile,
        };
        
        let index = Index::new(modes, chat_channels);
        
        Ok(CfgParam { config, index })
    }
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CharacterConfig {
    pub alias: String,
    pub id: CharacterId,
    pub intel_channel_ids: Vec<ChannelId>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CharacterCfg {
    pub alias: String,
    pub id: CharacterId,
    pub intel_channels: Vec<String>,
}

impl CharacterConfig {
    fn from_cfg(cfg: CharacterCfg, chat_channels: &ChatChannels) -> Self {
        let intel_channel_ids = cfg.intel_channels.into_iter()
            .map(|s| chat_channels.get_keyed(&s).expect("exists").id)
            .collect::<Vec<_>>();

        Self {
            alias: cfg.alias,
            id: cfg.id,
            intel_channel_ids,
        }
    }

    pub(crate) fn intel_channels<'a>(&self, chat_channels: &'a ChatChannels) -> Vec<&'a ChatChannel> {
        self.intel_channel_ids.iter()
            .map(|cid| chat_channels.get(*cid).expect("exists"))
            .collect()
    }
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CharactersCfg {
    pub character: Vec<CharacterCfg>,
}

impl CharactersCfg {
    const CHARACTERS_TOML: &'static str = "characters.toml";

    fn read(config_dir: &Path) -> SolarResult<Self> {
        let filepath = config_dir.join(Self::CHARACTERS_TOML);
        if !filepath.exists() {
            setup_config_file(&filepath, DEFAULT_CHARACTERS_TOML)?;
        }

        let toml_str = fs::read_to_string(&filepath)
            .map_err(|e| SolarError::read(e, &filepath))?;

        let toml: Self = toml::from_str(&toml_str)
            .map_err(|e| SolarError::cfg(e, filepath))?;

        Ok(toml)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SettingsCfg {
    pub logs_dir: PathBuf,
    pub modes: Vec<String>,
}

impl SettingsCfg {
    const SETTINGS_TOML: &'static str = "settings.toml";

    fn read(config_dir: &Path) -> SolarResult<Self> {
        let filepath = config_dir.join(Self::SETTINGS_TOML);
        if !filepath.exists() {
            setup_config_file(&filepath, DEFAULT_SETTINGS_TOML)?;
        }

        let toml_str = fs::read_to_string(&filepath)
            .map_err(|e| SolarError::read(e, &filepath))?;

        let toml: Self = toml::from_str(&toml_str)
            .map_err(|e| SolarError::cfg(e, filepath))?;

        Ok(toml)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ServerCfg {
    pub default: Option<String>,
    pub serve: Vec<ServeCfg>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ServeCfg {
    pub name: String,
    #[serde(flatten)]
    pub tls: TlsCfg,
}

pub(crate) trait CfgToml: serde::de::DeserializeOwned {
    const TOML_FILENAME: &'static str;
    const DEFAULT_TOML: &'static str;
    
    fn read(config_dir: &Path) -> SolarResult<Self> {
        let filepath = config_dir.join(Self::TOML_FILENAME);
        if !filepath.exists() {
            setup_config_file(&filepath, Self::DEFAULT_TOML)?;
        }

        let toml_str = fs::read_to_string(&filepath)
            .map_err(|e| SolarError::read(e, &filepath))?;

        let toml: Self = toml::from_str(&toml_str)
            .map_err(|e| SolarError::cfg(e, filepath))?;

        Ok(toml)
    }
}

impl CfgToml for ServerCfg {
    const TOML_FILENAME: &'static str = "server.toml";
    const DEFAULT_TOML: &'static str = DEFAULT_SERVER_TOML;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerProfileConfig {
    pub name: String,
    pub tls: TlsConfig,
}

impl ServerProfileConfig {
    fn try_from_cfg(cfg: ServeCfg) -> SolarResult<Self> {
        let tls = TlsConfig::try_from_cfg(cfg.tls)?;
        
        Ok(Self {
            name: cfg.name,
            tls,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClientCfg {
    pub default: Option<String>,
    // optional list of connections
    #[serde(default)]
    pub connect: Vec<ConnectCfg>,
}

impl CfgToml for ClientCfg {
    const TOML_FILENAME: &'static str = "client.toml";
    const DEFAULT_TOML: &'static str = DEFAULT_CLIENT_TOML;
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

impl ClientProfileConfig {
    fn try_from_cfg(cfg: ConnectCfg) -> SolarResult<Self> {
        let tls = TlsConfig::try_from_cfg(cfg.tls)?;
        
        Ok(Self {
            name: cfg.name,
            tls,
        })
    }
}

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

fn setup_config_file(filepath: &Path, defaults: &str) -> SolarResult<()> {
    let logpath = log_path(&filepath);
    println!("{INFO} configuring {logpath}");
    fs::write(&filepath, defaults)
        .map_err(|e| SolarError::write(e, &filepath))?;

    if let Ok(editor_cmd) = env::var("EDITOR") {
        let mut cmd = Command::new(editor_cmd);
        cmd.arg(&filepath);
        let status = cmd.status()
            .map_err(|_| SolarError::msg("Unable to run environment $EDITOR"))?;

        match status.success() {
            true => Ok(()),
            false => SolarError::err_msg("{ERR} failed to configure {logpath}"),
        }
    } else {
        SolarError::err_msg("{ERR} please configure {logpath}")
    }
}

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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq,
    serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode
)]
pub enum PingKind {
    Combat,
    Danger,
    Intel,
    Message,
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

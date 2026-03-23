use crate::*;

pub(crate) const CONFIG_DIR: &'static str = ".config/solar_sonar";
pub(crate) const CERTS_DIR: &'static str = "certs";

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
        let logs_dir = shellexpand::path::full(&self.settings.logs_dir)
            .map_err(|_| SolarError::msg(format!("Environment expansion failed for settings.log_dir: {}", self.settings.logs_dir.to_string_lossy())))?;
        
        let logs_dir = match logs_dir {
            Cow::Borrowed(p) => p.into(),
            Cow::Owned(p) => p,
        };
        
        let modes = ModeIndex::try_new(self.settings.modes)?;
        
        //todo: pull from logs.toml
        let channel_names = self.characters.iter()
            .map(|chr| &chr.intel_channels)
            .flatten()
            .map(String::clone)
            .collect::<Vec<_>>();
        
        let chat_channels = ChatChannelIndex::try_new(channel_names)?;
        
        
        let characters = self.characters.into_iter()
            .map(|chr| CharacterConfig::try_from_cfg(chr))
            .collect::<Vec<_>>();
        
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

pub(crate) fn setup_config_file(filepath: &Path, defaults: &str) -> SolarResult<()> {
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

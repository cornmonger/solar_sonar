use std::collections::HashSet;

use crate::*;

const CONFIG_DIR: &'static str = ".config/solar_sonar";
const DEFAULT_CHARACTERS_TOML: &'static str = include_str!("../assets/config/default/characters.toml");
const DEFAULT_SETTINGS_TOML: &'static str = include_str!("../assets/config/default/settings.toml");

// Fully processing configuration. Consumed internally. Available publically
// primarily for inspection.
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub characters: Vec<CharacterConfig>,
    pub logs_dir: PathBuf,
}

/// Public-facing configuration. Converted into [Config] at init.
/// Call [Cfg::read], [Cfg::read_dir], or [Cfg::build] to prepare.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Cfg {
    pub characters: Vec<CharacterCfg>,
    pub settings: SettingsCfg,
}

/// The result of building a [Cfg].
#[derive(Debug, PartialEq, Eq)]
pub struct CfgParam {
    pub config: Config,
    pub chat_channels: ChatChannels,
}

impl Cfg {
    pub fn read() -> SolarResult<CfgParam> {
        let config_dir = SolarSonar::get().dirs.home_dir().join(CONFIG_DIR);
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

        Cfg { characters, settings }.build()
    }

    pub fn build(self) -> SolarResult<CfgParam> {
        let channel_names = self.characters.iter()
            .map(|chr| &chr.intel_channels)
            .flatten()
            .map(String::clone)
            .collect::<Vec<_>>();
        
        let chat_channels = ChatChannels::new(channel_names);
        let characters = self.characters.into_iter()
            .map(|chr| CharacterConfig::from_cfg(chr, &chat_channels))
            .collect::<Vec<_>>();

        let logs_dir = shellexpand::path::full(&self.settings.logs_dir)
            .map_err(|_| SolarError::msg(format!("Environment expansion failed for settings.log_dir: {}", self.settings.logs_dir.to_string_lossy())))?;
        
        let logs_dir = match logs_dir {
            Cow::Borrowed(p) => p.into(),
            Cow::Owned(p) => p,
        };

        Ok(CfgParam {
            config: Config { characters, logs_dir },
            chat_channels
        })
    }
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CharacterConfig {
    pub alias: String,
    pub id: CharacterID,
    pub intel_channel_ids: Vec<ChatChannelId>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CharacterCfg {
    pub alias: String,
    pub id: CharacterID,
    pub intel_channels: Vec<String>,
}

impl CharacterConfig {
    fn from_cfg(cfg: CharacterCfg, chat_channels: &ChatChannels) -> Self {
        let intel_channel_ids = cfg.intel_channels.into_iter()
            .map(|s| chat_channels.find_name(&s).expect("exists").id)
            .collect::<Vec<_>>();

        Self {
            alias: cfg.alias,
            id: cfg.id,
            intel_channel_ids,
        }
    }

    pub(crate) fn intel_channels<'a>(&self, chat_channels: &'a ChatChannels) -> Vec<ChatChannelRef<'a>> {
        self.intel_channel_ids.iter()
            .map(|cid| chat_channels.find_id(*cid).expect("exists"))
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

pub type ChatChannelId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatChannelRef<'a> {
    pub name: &'a str,
    pub id: ChatChannelId,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChatChannel {
    pub name: String,
    pub id: ChatChannelId,
}

impl std::hash::Hash for ChatChannel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<'a> From<&'a ChatChannel> for ChatChannelRef<'a> {
    fn from(v: &'a ChatChannel) -> Self {
        Self {
            name: v.name.as_str(),
            id: v.id,
        }
    }
}

impl<'a> Display for ChatChannelRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChatChannels(HashSet<ChatChannel>);
impl ChatChannels {
    pub fn new(channels: Vec<String>) -> Self {
        let channels = channels.into_iter()
            .map(|s| ChatChannel {
                id: xxh3_64(s.as_bytes()),
                name: s,
            })
            .collect::<HashSet<_>>();

        Self(channels)
    }

    pub fn extend(&mut self, channels: ChatChannels) {
        self.0.extend(channels.0);
    }

    pub fn find_id(&self, id: ChatChannelId) -> Option<ChatChannelRef<'_>> {
        self.0.iter().find(|c| c.id == id).map(ChatChannelRef::from)
    }

    pub fn find_name(&self, name: &str) -> Option<ChatChannelRef<'_>> {
        self.0.iter().find(|c| c.name == name).map(ChatChannelRef::from)
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<'_, ChatChannel> {
        self.0.iter()
    }
}


use crate::*;

const CONFIG_DIR: &'static str = ".config/solar_sonar";

const DEFAULT_CHARACTERS_TOML: &'static str = include_str!("../assets/config/default/characters.toml");
const DEFAULT_SETTINGS_TOML: &'static str = include_str!("../assets/config/default/settings.toml");

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub characters: Vec<CharacterConfig>,
    pub logs_dir: PathBuf,
}

impl Config {
    pub(crate) fn read() -> SolarResult<Self> {
        let config_dir = RunSys::get().dirs.home_dir().join(CONFIG_DIR);
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| SolarError::mkdir(e, &config_dir))?;
        }

        let settings = SettingsToml::read(&config_dir)?;
        let characters = CharactersToml::read(&config_dir)?
            .character.into_iter()
            .map(|c| c.into())
            .collect::<Vec<_>>();
        let logs_dir = shellexpand::path::full(&settings.logs_dir)
            .map_err(|_| SolarError::msg(format!("Environment expansion failed for settings.log_dir: {}", settings.logs_dir.to_string_lossy())))?;
        
        let logs_dir = match logs_dir {
            Cow::Borrowed(p) => p.into(),
            Cow::Owned(p) => p,
        };

        Ok(Self {
            characters,
            logs_dir,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CharacterConfig {
    pub alias: String,
    pub id: CharacterID,
    pub id_str: String,
    pub intel_channels: Vec<String>,
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CharacterCfg {
    pub alias: String,
    pub id: CharacterID,
    pub intel_channels: Vec<String>,
}

impl From<CharacterCfg> for CharacterConfig {
    fn from(v: CharacterCfg) -> Self {
        Self {
            alias: v.alias,
            id: v.id,
            id_str: v.id.to_string(),
            intel_channels: v.intel_channels,
        }
    }
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CharactersToml {
    pub character: Vec<CharacterCfg>,
}

impl CharactersToml {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SettingsToml {
    logs_dir: PathBuf,
}

impl SettingsToml {
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


use crate::*;

#[derive(Debug, Clone, clap::Parser)]
pub struct Cli {
    /// Characters to watch intel for
    pub watch_characters: String,
    /// Systems to watch
    pub watch_systems: String,
    /// Jump range of systems
    #[clap(default_value_t = Args::DEFAULT_JUMPS)]
    pub jumps: u8,
    /// Enable / disable output to terminal
    #[clap(long, default_value_t = Args::DEFAULT_STDIO, action = clap::ArgAction::Set)]
    pub stdio: bool,
    // Enable / disable audio alerts
    #[clap(long, default_value_t = Args::DEFAULT_AUDIO, action = clap::ArgAction::Set)]
    pub audio: bool,
    /// Replay a specific log file
    #[clap(long)]
    pub replay: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Args {
    pub watch_character_ids: Vec<CharacterID>,
    pub watch_system_ids: Vec<SolarID>,
    pub jumps: u8,
    pub audio: bool,
    pub stdio: bool,
    pub replay_file: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ArgParam {
    pub args: Args,
    pub chat_channels: Option<ChatChannels>,
}

impl Args {
    pub const DEFAULT_JUMPS: u8 = 2;
    pub const DEFAULT_AUDIO: bool = true;
    pub const DEFAULT_STDIO: bool = true;
}

impl Args {
    pub(crate) fn try_from_cli(cli: Cli, cfg: &Config) -> SolarResult<ArgParam> {
        let jumps = cli.jumps;
        let watch_character_ids = cli.watch_characters.split(',')
            .map(|snake| snake.to_snake_case())
            .map(|snake| cfg.characters.iter()
                .find(|chr| chr.alias == snake)
                .map(|chr| chr.id)
                .ok_or_else(|| SolarError::msg(format!("Character unknown: {snake}")))
            )
            .collect::<SolarResult<Vec<_>>>()?;

        let watch_system_ids = cli.watch_systems.split(',')
            .map(|name| name.to_uppercase())
            .map(|ref name| STAR_MAP.get_system_named(name)
                .map(|sys| sys.id)
                .ok_or_else(|| SolarError::msg(format!("System unknown: {name}")))
            )
            .collect::<SolarResult<Vec<_>>>()?;

        let replay_file = cli.replay
            .and_then(|p| Some(expand_pathbuf(p)))
            .transpose()?;

        let this = Args {
            watch_character_ids,
            watch_system_ids,
            jumps,
            audio: cli.audio,
            stdio: cli.stdio,
            replay_file,
        };

        this.build()
    }

    pub fn build(self) -> SolarResult<ArgParam> {
        let chat_channels = self.replay_file.as_ref().map(PathBuf::from)
            .and_then(|f| ChatLogFile::from_path_buf(f))
            .map(|f| ChatChannels::new(vec![f.channel().to_string()]));

        Ok(ArgParam {
            args: self,
            chat_channels,
        })
    }
}

impl Args {
    pub(crate) fn watch_characters<'a>(&self, cfg: &'a Config) -> Vec<&'a CharacterConfig> {
        self.watch_character_ids.iter()
            .filter_map(|id| cfg.characters.iter().find(|chr| &chr.id == id))
            .collect()
    }

    pub(crate) fn watch_systems(&self) -> Vec<&'static SolarSystem > {
        self.watch_system_ids.iter()
            .map(|id| STAR_MAP.system(id))
            .collect()
    }

    pub(crate) fn is_replay(&self) -> bool {
        self.replay_file.is_some()
    }
}


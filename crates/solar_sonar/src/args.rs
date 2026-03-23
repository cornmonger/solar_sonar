use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Args {
    pub watch_character_ids: Vec<CharacterId>,
    pub watch_system_ids: Vec<SolarId>,
    pub jumps: u8,
    pub audio: bool,
    pub stdio: bool,
    pub server_profile: Option<String>,
    pub client_profile: Option<String>,
    pub replay_file: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ArgParam {
    pub args: Args,
    pub chat_channels: Option<ChatChannelIndex>,
}

impl Args {
    pub const DEFAULT_JUMPS: u8 = 2;
    pub const DEFAULT_AUDIO: bool = true;
    pub const DEFAULT_STDIO: bool = true;
    
    pub(crate) fn try_from_cli(cli: Cli, cfg: &Config) -> SolarResult<ArgParam> {
        if cli.connect.is_some() && cli.serve.is_some() {
            return SolarError::err_msg("Cannot both serve and connect");
        }
        
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
            server_profile: cli.serve,
            client_profile: cli.connect,
            replay_file,
        };

        this.build()
    }

    pub fn build(self) -> SolarResult<ArgParam> {
        let chat_channels = self.replay_file.as_ref().map(PathBuf::from)
            .and_then(|f| ChatLogFile::from_path_buf(f))
            .map(|f| ChatChannelIndex::try_new(vec![f.channel().to_string()]))
            .transpose()?;

        Ok(ArgParam {
            args: self,
            chat_channels,
        })
    }
    
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

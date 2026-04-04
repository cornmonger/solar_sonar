use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Args {
    pub mode: String,
    pub watch_character_ids: Vec<CharacterId>,
    pub watch_system_ids: Vec<StarId>,
    pub jumps: u8,
    pub audio: bool,
    pub stdio: bool,
    pub server_profile: Option<String>,
    pub client_profile: Option<String>,
    pub replay_path: Option<PathBuf>,
    pub relog: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ArgParam {
    pub args: Args,
    pub(crate) mode_id: ModeId,
    pub(crate) arg_indexed_logs: Vec<IndexedLog>,
}

impl Args {
    pub const DEFAULT_JUMPS: u8 = 2;
    pub const DEFAULT_AUDIO: bool = true;
    pub const DEFAULT_STDIO: bool = true;
    
    pub(crate) fn try_from_cli(cli: Cli, cfg: &Config, index:  &mut Index) -> SolarResult<ArgParam> {
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
        
        let binlog = cli.relog
            .and_then(|p| Some(expand_pathbuf(p)))
            .transpose()?;

        let this = Args {
            mode: cli.mode,
            watch_character_ids,
            watch_system_ids,
            jumps,
            audio: cli.audio,
            stdio: cli.stdio,
            server_profile: cli.serve,
            client_profile: cli.connect,
            replay_path: replay_file,
            relog: binlog,
        };

        this.build(index)
    }

    pub fn build(self, index: &mut Index) -> SolarResult<ArgParam> {
        let arg_indexed_logs = if let Some(replay_file) = self.replay_path.as_ref().map(PathBuf::from) {
            let dir_kind = LogDirKind::from_file_path(&replay_file)
                .unwrap_or(LogDirKind::Chat);
            let log_file = LogFile::from_path_buf(replay_file, dir_kind)
                .ok_or_else(|| SolarError::msg("Invalid replay log file"))?;
            let log_name = NamedLog::from_log_file(log_file)?;
            let channel_names = index.chat_channels_mut();
            let indexed_log_name = channel_names.index_named(log_name);
            vec![indexed_log_name]
        } else {
            vec![]
        };
        
        let mode_id = index.modes().find(&self.mode)?.id();

        Ok(ArgParam {
            args: self,
            arg_indexed_logs,
            mode_id,
        })
    }
    
    pub(crate) fn watch_characters<'a>(&self, cfg: &'a Config) -> Vec<&'a CharacterConfig> {
        self.watch_character_ids.iter()
            .filter_map(|id| cfg.characters.iter().find(|chr| &chr.id == id))
            .collect()
    }
    
    pub(crate) fn watch_systems(&self) -> Vec<&'static StarSystem > {
        self.watch_system_ids.iter()
            .map(|id| STAR_MAP.system(id))
            .collect()
    }

    pub(crate) fn is_replay(&self) -> bool {
        self.replay_path.is_some()
    }
}

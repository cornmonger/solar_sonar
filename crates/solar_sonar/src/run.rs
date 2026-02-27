use crate::*;

pub fn run() -> ExitCode {
    if handle_error(SolarSonar::init_once()).is_err() {
        return ExitCode::FAILURE;
    }
    
    let cli = Cli::parse();
    let Ok(CfgParam{config, mut chat_channels}) = handle_error(Cfg::read()) else {
        return ExitCode::FAILURE
    };
    let Ok(args_param) = handle_error(Args::try_from_cli(cli, &config)) else {
        return ExitCode::FAILURE
    };

    let ArgParam{args, chat_channels: args_chat_channels} = args_param;
    let arg_channel_ids;
    if let Some(args_chat_channels) = args_chat_channels {
        arg_channel_ids = args_chat_channels.iter().map(|c| c.id).collect::<Vec<_>>();
        chat_channels.extend(args_chat_channels);
    } else {
        arg_channel_ids = vec![];
    }

    let run = Running { args, cfg: config, chat_channels, arg_channel_ids, stdio: Some(StdIO::default()), event_io: None };

    match handle_error(run_cli(run)) {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

pub fn start(arg: ArgParam, cfg: CfgParam, io: EventIO) -> SolarResult<tokio::task::JoinHandle<SolarResult<()>>> {
    handle_error(SolarSonar::init_once())?;

    let ArgParam { args, chat_channels: args_chat_channels } = arg;
    let CfgParam { config, mut chat_channels } = cfg;

    let stdio = match args.stdio {
        true => Some(StdIO::default()),
        false => None,
    };

    let arg_channel_ids;
    if let Some(args_chat_channels) = args_chat_channels {
        arg_channel_ids = args_chat_channels.iter().map(|c| c.id).collect::<Vec<_>>();
        chat_channels.extend(args_chat_channels);
    } else {
        arg_channel_ids = vec![];
    }

    let run = Running { args, cfg: config, chat_channels, arg_channel_ids, stdio, event_io: Some(io) };
    
    let handle = tokio::task::spawn_blocking(move || {
        handle_error(run_cli(run))
    });

    Ok(handle)
}

fn handle_error<T>(result: SolarResult<T>) -> SolarResult<T> {
    match result {
        Ok(t) => Ok(t),
        Err(e) => {
            eprintln!("{ERR} {e}");
            if let Some(source) = e.source() {
                eprintln!("{ERR} {source}");
            }

            Err(e)
        },
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Event {
    Downloading {
        id: u8,
    },
    Download {
        id: u8,
        success: bool,
    },
    Args {
        system_ids: Vec<SolarID>,
        character_ids: Vec<CharacterID>,
        channels: Vec<String>,
        jumps: u8,
        system_range: Vec<SolarID>,
    },
    LogEntry {
        entry: LogEntry,
    },
    PingFortune,
    PingSystems {
        system_ids: Vec<SolarID>,
    },
}

fn run_cli(run: Running) -> SolarResult<()> {
    for asset in RemoteAssets::all() {
        let filepath = asset.filepath()?;
        if filepath.exists() { continue }

        if run.stdio.is_some() {
            print!("{INFO} downloading asset {MAGENTA}{}{CLR} ... ", asset.name);
            let _ = io::stdout().flush();
        }
        if let Some(event_io) = &run.event_io {
            event_io.send(Event::Downloading { id: asset.id })?;
        }

        match asset.download() {
            Ok(_) => {
                if run.stdio.is_some() {
                    println!("{GREEN}done{CLR}");
                }
                if let Some(event_io) = &run.event_io {
                    event_io.send(Event::Download { id: asset.id, success: true })?;
                }
            },
            Err(e) => {
                if run.stdio.is_some() {
                    println!("{RED}failed{CLR}");
                }
                if let Some(event_io) = &run.event_io {
                    event_io.send(Event::Download { id: asset.id, success: false })?;
                }
                return Err(e);
            },
        }
    }

    const ENV_ESPEAK_DATA_PATH: &'static str = "ESPEAK_DATA_PATH";
    if env::var(ENV_ESPEAK_DATA_PATH).is_err() {
        unsafe {
            env::set_var(ENV_ESPEAK_DATA_PATH, RemoteAssets::EspeakData.get().dirpath()?)
        }
    }
    
    let watch_channels = run.watch_channels();

    let nav = StarNavigator::default();
    let watch_range = run.args.watch_systems().iter()
        .map(|sys| nav.systems_in_range(run.args.jumps, sys))
        .flatten()
        .collect::<Vec<_>>();

    if run.stdio.is_some() {
        let systems = run.args.watch_systems().into_iter()
            .map(|sys| format!("{BRIGHT_BLUE}{}{CLR}", sys.name) )
            .collect::<Vec<_>>()
            .join(" ");
        let characters = format!("{MAGENTA}{}{CLR}",
            run.args.watch_characters(&run.cfg).into_iter()
                .map(|c| c.alias.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        let channels = watch_channels.iter()
            .map(|s| format!("{GREEN}{s}{CLR}"))
            .collect::<Vec<_>>()
            .join(" ");
        let range = watch_range.iter()
            .map(|sys| format!("{BRIGHT_BLUE}{}{CLR}", sys.name))
            .collect::<Vec<_>>()
            .join(" ");
        
        println!("{INFO} watching {systems} for {characters} in {channels}");
        println!("{INFO} within {} jumps: {range}", run.args.jumps);
    }
    if let Some(event_io) = &run.event_io {
        event_io.send(Event::Args {
            system_ids: run.args.watch_system_ids.clone(),
            character_ids: run.args.watch_character_ids.clone(),
            channels: watch_channels.iter().map(|s| s.name.to_string()).collect(),
            jumps: run.args.jumps,
            system_range: watch_range.iter().map(|sys| sys.id).collect(),
        })?;
    }
    
    let (mut last_stamp, replay_log) = match &run.args.replay_file {
        Some(p) => {
            let log = ChatLogFile::from_path_buf(p.to_path_buf())
                .ok_or_else(|| SolarError::msg(format!("Invalid chat log: {}", log_path(p))))?;

            (log.datetime().clone(), Some(log))
        },
        None => (Utc::now(), None),
    };

    if run.args.audio {
        play_ping(vec![fortune()])?;
    }

    let mut logs = ChannelLogs::new_watch(&run);

    loop {
        logs = match &replay_log {
            None => read_intel_logs(&run, logs)?,
            Some(logfile) => {
                let read = read_intel_log_file(&logfile.path(), 0)?;
                logs.push(&run, &logfile, read);
                logs
            },
        };

        let stamp = last_stamp;
        let activity = watch_channels.iter()
            .filter_map(|channel| logs.get(channel.id))
            .flat_map(|log| log.entries.iter())
            .filter(|entry| entry.datetime > stamp)
            .filter(|entry| !entry.analysis.systems.is_empty())
            .collect::<Vec<_>>();

        for entry in activity {
            last_stamp = entry.datetime;
            let matched_systems = entry.analysis.systems.iter()
                .filter_map(|sys_id| watch_range.iter().find(|sys| &sys.id == sys_id))
                .collect::<Vec<_>>();

            let in_range = !matched_systems.is_empty();
            let danger = entry.analysis.danger();
            
            if run.stdio.is_some() {
                println!("{}", entry.display_ansi(in_range, danger));
            }
            if let Some(event_io) = &run.event_io {
                event_io.send(Event::LogEntry { entry: entry.clone() })?;
            }

            if !in_range {
                continue;
            }

            let names = matched_systems.iter()
                .map(|sys| sys.name)
                .collect::<Vec<_>>();
    
            if danger {
                if let Some(event_io) = &run.event_io {
                    event_io.send(Event::PingSystems {
                        system_ids: matched_systems.iter().map(|sys| sys.id).collect()
                    })?;
                }

                if run.args.audio {
                    play_ping_systems(names)?;
                }
            }
        }

        if run.args.is_replay() {
            return Ok(());
        } else if stamp == last_stamp {
            // sleep if there hasn't been activity 
            std::thread::sleep(Duration::from_secs(2))
        }
    }
}

#[derive(Debug)]
pub(crate) struct Running {
    pub(crate) args: Args,
    pub(crate) arg_channel_ids: Vec<ChatChannelId>,
    pub(crate) chat_channels: ChatChannels,
    pub(crate) cfg: Config,
    pub(crate) stdio: Option<StdIO>,
    pub(crate) event_io: Option<EventIO>,
}

impl Running {
    pub(crate) fn watch_channels(&self) -> Vec<ChatChannelRef<'_>> {
        let arg_channels = self.arg_channel_ids.iter()
            .map(|id| self.chat_channels.find_id(*id).expect("exists"));

        self.args.watch_characters(&self.cfg).iter()
            .map(|chr| chr.intel_channels(&self.chat_channels))
            .flatten()
            .chain(arg_channels)
            .collect()
    }
}

#[derive(Debug, Default)]
pub(crate) struct StdIO;

#[derive(Debug)]
pub struct EventIO {
    pub(crate) tx: EventTx,
}

pub type EventTx = tokio::sync::mpsc::UnboundedSender<Event>;
pub type EventRx = tokio::sync::mpsc::UnboundedReceiver<Event>;

impl EventIO {
    pub(crate) fn new() -> (Self, EventRx) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        ( Self { tx }, rx )
    }

    pub(crate) fn send(&self, event: Event) -> SolarResult<()> {
        self.tx.send(event)
            .map_err(|e| SolarError::send(e))
    }
}

pub struct SolarSonar {
    pub(crate) dirs: directories::BaseDirs,
}

impl SolarSonar {
    fn init() -> SolarResult<Self> {
        let dirs = directories::BaseDirs::new()
            .ok_or_else(|| SolarError::msg("Failed to load system directories"))?;

        Ok(Self {
            dirs,
        })
    }

    pub fn init_once() -> SolarResult<&'static SolarSonar> {
        let runsys = SolarSonar::init()?;
        Ok(RUN_SYS.get_or_init(|| runsys))
    }

    pub(crate) fn get() -> &'static Self {
        RUN_SYS.get_or_init(|| panic!("System not initialized"))
    }

    pub fn make_io() -> (EventIO, EventRx) {
        EventIO::new()
    }
}

pub(crate) static RUN_SYS: OnceLock<SolarSonar> = OnceLock::new();

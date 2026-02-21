use crate::*;

pub fn run() -> ExitCode {
    if handle_error(RunSys::init_once()).is_err() {
        return ExitCode::FAILURE;
    }
    
    let cli = Cli::parse();
    let Ok(cfg) = handle_error(Config::read()) else {
        return ExitCode::FAILURE
    };
    let Ok(args) = handle_error(Args::try_from_cli(cli, &cfg)) else {
        return ExitCode::FAILURE
    };

    let run = Running { args, cfg };
    match handle_error(run_cli(run)) {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

pub fn run_with(args: Args, cfg: Config) -> SolarResult<()> {
    handle_error(RunSys::init_once())?;
    let run = Running { args, cfg };
    handle_error(run_cli(run))
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

fn run_cli(run: Running) -> SolarResult<()> {
    for asset in RemoteAssets::all() {
        let filepath = asset.filepath()?;
        if filepath.exists() { continue }
        print!("{INFO} downloading asset {MAGENTA}{}{CLR} ... ", asset.name);
        let _ = io::stdout().flush();
        match asset.download() {
            Ok(_) => println!("{GREEN}done{CLR}"),
            Err(e) => {
                println!("{RED}failed{CLR}");
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
    
    let watch_channels = run.args.watch_characters(&run.cfg).iter()
        .map(|chr| &chr.intel_channels)
        .flatten()
        .collect::<Vec<_>>();

    let nav = StarNavigator::default();
    let watch_range = run.args.watch_systems().iter()
        .map(|sys| nav.systems_in_range(run.args.jumps, sys))
        .flatten()
        .collect::<Vec<_>>();

    {
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

    play_ping(vec![fortune()])?;
    
    let mut logs = ChannelLogs::new_watch(&run);
    let mut last_stamp = Utc::now();

    loop {
        logs = read_intel_logs(&run, logs)?;
        let stamp = last_stamp;
        let activity = watch_channels.iter()
            .filter_map(|channel| logs.get(channel))
            .flat_map(|log| log.entries.iter())
            .filter(|entry| entry.datetime > stamp)
            .filter(|entry| !entry.analysis.systems.is_empty())
            .collect::<Vec<_>>();

        for entry in activity {
            last_stamp = entry.datetime;
            let matched_systems = entry.analysis.systems.iter()
                .filter_map(|sys_id| watch_range.iter().find(|sys| &sys.id == sys_id))
                .collect::<Vec<_>>();

            let alert = !matched_systems.is_empty();
            println!("{}", entry.display_ansi(alert));

            if !alert {
                continue;
            }

            let names = matched_systems.iter()
                .map(|sys| sys.name)
                .collect::<Vec<_>>();

            if !entry.analysis.keywords.contains(&LogKeyword::Clear) {
                play_ping_systems(names)?;
            }
        }

        // sleep if there hasn't been activity 
        if stamp == last_stamp {
            std::thread::sleep(Duration::from_secs(2))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Running {
    pub args: Args,
    pub cfg: Config,
}

pub(crate) struct RunSys {
    pub(crate) dirs: directories::BaseDirs,
}

impl RunSys {
    fn init() -> SolarResult<Self> {
        let dirs = directories::BaseDirs::new()
            .ok_or_else(|| SolarError::msg("Failed to load system directories"))?;

        Ok(Self {
            dirs,
        })
    }

    fn init_once() -> SolarResult<&'static RunSys> {
        let runsys = RunSys::init()?;
        Ok(RUN_SYS.get_or_init(|| runsys))
    }

    pub(crate) fn get() -> &'static Self {
        RUN_SYS.get_or_init(|| panic!("System not initialized"))
    }
}

pub(crate) static RUN_SYS: OnceLock<RunSys> = OnceLock::new();

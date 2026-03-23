use futures::future::BoxFuture;

use crate::*;

pub async fn run() -> ExitCode {
    if handle_error(SolarSonar::init_once()).is_err() {
        return ExitCode::FAILURE;
    }
    
    let cli = Cli::parse();
    let Ok(CfgParam{config, mut index}) = handle_error(Cfg::read()) else {
        return ExitCode::FAILURE
    };
    let Ok(args_param) = handle_error(Args::try_from_cli(cli, &config)) else {
        return ExitCode::FAILURE
    };

    let ArgParam{args, chat_channels: args_chat_channels} = args_param;
    let arg_channel_ids;
    if let Some(args_chat_channels) = args_chat_channels {
        arg_channel_ids = args_chat_channels.iter().map(|c| c.id).collect::<Vec<_>>();
        if handle_error(index.chat_channels_mut().extend(args_chat_channels)).is_err() {
            return ExitCode::FAILURE;
        }
    } else {
        arg_channel_ids = vec![];
    }
    
    let sonar_options = SonarOptions {
        audio: args.audio,
        stdio: args.stdio,
    };

    if check_args(&args, &config).is_err() {
        return ExitCode::FAILURE;
    }
    
    let params = RunningParams {
        args,
        cfg: config,
        index,
        arg_channel_ids,
        io: sonar_options,
    };
    
    let Ok(Startup { running, sonar_io }) = handle_error(Running::startup(params)) else {
        return ExitCode::FAILURE;
    };

    match handle_error(run_cli(running, sonar_io).await) {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn check_args(args: &Args, config: &Config) -> SolarResult<()> {
    handle_error(check_server_profile(&args, &config))
}


fn check_server_profile(args: &Args, cfg: &Config) -> SolarResult<()> {
    if let Some(serve_name) = args.server_profile.as_deref() {
        find_server_profile(args, cfg)
            .map(|_| ())
            .ok_or_else(|| SolarError::msg(format!("Server profile not found: {serve_name}")))
    } else {
        Ok(())
    }
}

pub struct SolarSonarHandle {
    cancel: r::tokio::CancellationToken,
    handle: r::tokio::JoinHandle<SolarResult<()>>,
    rx: SonarBroadcastRx,
}

impl SolarSonarHandle {
    pub async fn recv(&mut self) -> SolarResult<DataEvent> {
        self.rx.recv().await
            .map_err(|_e| SolarError::msg("closed"))
    }
    
    pub async fn close(self) {
        if !self.cancel.is_cancelled() {
            self.cancel.cancel();
        }
        if !self.handle.is_finished() {
            let _ = r::tokio::timeout(Duration::from_secs(20), async move {
                let _ = self.handle.await;
            }).await;
        }
    }
}

pub fn start(arg: ArgParam, cfg: CfgParam) -> SolarResult<SolarSonarHandle> {
    handle_error(SolarSonar::init_once())?;

    let ArgParam { args, chat_channels: args_chat_channels } = arg;
    let CfgParam { config, mut index } = cfg;

    let sonar_options = SonarOptions {
        audio: args.audio,
        stdio: args.stdio,
    };
    
    let arg_channel_ids;
    if let Some(args_chat_channels) = args_chat_channels {
        arg_channel_ids = args_chat_channels.iter().map(|c| c.id).collect::<Vec<_>>();
        index.chat_channels_mut().extend(args_chat_channels)?;
    } else {
        arg_channel_ids = vec![];
    }

    check_args(&args, &config)?;
    
    let run = RunningParams {
        args,
        cfg: config,
        index,
        arg_channel_ids,
        io: sonar_options,
    };
    
    let Startup { running, sonar_io } = handle_error(Running::startup(run))?;
    let rx = sonar_io.subscribe();
    
    let cancel = r::tokio::CancellationToken::new();
    let handle = tokio::task::spawn(async move {
        handle_error(run_cli(running, sonar_io).await)
    });
    
    
    let handle = SolarSonarHandle {
        cancel,
        handle,
        rx,
    };

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

async fn run_cli(run: Running, mut io: SonarIO) -> SolarResult<()> {
    for asset in RemoteAssets::all() {
        let filepath = asset.filepath()?;
        if filepath.exists() { continue }
        
        io.broadcast(DataEvent::Downloading { id: asset.id })?;

        match asset.download() {
            Ok(_) => {
                io.broadcast(DataEvent::Download { id: asset.id, success: true })?;
            },
            Err(e) => {
                io.broadcast(DataEvent::Download { id: asset.id, success: false })?;
                return Err(e);
            },
        }
    }

    const ENV_ESPEAK_DATA_PATH: &'static str = "ESPEAK_DATA_PATH";
    if env::var(ENV_ESPEAK_DATA_PATH).is_err() {
        unsafe {
            env::set_var(ENV_ESPEAK_DATA_PATH, RemoteAssets::EspeakData.asset().dirpath()?)
        }
    }
    
    let watch_channels = run.watch_logs();
    let watch_range = run.watch_range();

    io.broadcast(DataEvent::Args(ArgsData {
        system_ids: run.args.watch_system_ids.clone(),
        character_ids: run.args.watch_character_ids.clone(),
        channels: watch_channels.clone(),
        jumps: run.args.jumps,
        system_range: watch_range.iter().map(|sys| sys.id).collect(),
    }))?;

    let tls_server = if let Some(cfg) = run.server_profile() {
        let cert_files = CertFiles::new(&SolarSonar::get().config_dir().join(CERTS_DIR), "server");
        if !cert_files.exist() {
            io.broadcast(DataEvent::GeneratingCerts)?;
            
            match init_certs(&cert_files, "my_server") {
                Ok(_) => io.broadcast(DataEvent::GenerateCerts { success: true })?,
                Err(e) => {
                    io.broadcast(DataEvent::GenerateCerts { success: false })?;
                    return Err(e);
                },
            }
        }
                
        Some(TlsServer::start(&run, TlsServerOptions {
            ip: cfg.tls.ip,
            port: cfg.tls.port,
            cert_files,
            event_rx: io.subscribe() 
        }).await?)
    } else {
        None
    };
    
    let mut tls_client = if let Some(cfg) = run.client_profile() {
        let cert_files = CertFiles::new(&SolarSonar::get().config_dir().join(CERTS_DIR), "server");
        if !cert_files.exist() {
            return SolarError::err_msg("TLS server certificates do not exist"); // todo bad
        }
            
        Some(TlsClient::start(&run, &io, TlsClientOptions {
            ip: cfg.tls.ip,
            port: cfg.tls.port,
            cert_files,
        }).await?)
    } else {
        None
    };
    
    let source_kind = if run.args.client_profile.is_some() {
        SourceKind::Client
    } else if run.args.replay_file.is_some() {
        SourceKind::Replay
    } else {
        SourceKind::Logs
    };
    
    let (mut last_stamp, replay_log) = match &run.args.replay_file {
        Some(p) => {
            let log = ChatLogFile::from_path_buf(p.to_path_buf())
                .ok_or_else(|| SolarError::msg(format!("Invalid chat log: {}", log_path(p))))?;

            (log.timestamp().clone(), Some(log))
        },
        None => (Utc::now().into(), None),
    };

    io.broadcast(DataEvent::PingFortune)?;

    let mut logs = Logs::new_watch(&run);
    let mut sleep_time = Duration::from_secs(0);
    let signal_ctl_c = tokio::signal::ctrl_c();
    tokio::pin!(signal_ctl_c);
    
    loop {
        let stamp = last_stamp;
        
        let source: BoxFuture<SolarResult<Vec<LogEntry>>> = match source_kind {
            SourceKind::Logs => {
                let run = &run;
                let watch_channels = &watch_channels;
                let logs = &mut logs;
                Box::pin(async move { select_logs(&run, &watch_channels, stamp, logs).await })
            },
            SourceKind::Replay => {
                let run = &run;
                let watch_channels = &watch_channels;
                let replay_log = replay_log.as_ref().expect("exists");
                let logs = &mut logs;
                Box::pin(async move { select_replay(&run, &watch_channels, stamp, logs, &replay_log, 1).await })
            },
            SourceKind::Client => {
                let tls_client = &mut tls_client;
                Box::pin(async move { select_client(stamp, tls_client).await })
            },
        };
        
        let activity = tokio::select! {
            _ = &mut signal_ctl_c => break,
            result = io.select(&run) => match result {
                Ok(_) => continue,
                Err(_e) => break,
            },
            result = source => match result {
                Ok(activity) => activity,
                Err(_) => break,
            },
            _ = tokio::time::sleep(sleep_time) => continue,
        };
        
        for entry in activity {
            last_stamp = entry.timestamp;
            let matched_systems = entry.analysis.system_ids().iter()
                .filter_map(|sys_id| watch_range.iter().find(|sys| &sys.id == sys_id))
                .collect::<Vec<_>>();

            let in_range = !matched_systems.is_empty();
            let in_danger = entry.analysis.in_danger();
            
            io.broadcast(DataEvent::LogEntry { entry: entry.clone(), in_range, in_danger })?;

            if !in_range {
                continue;
            }

            if in_danger {
                io.broadcast(DataEvent::PingSystems {
                    system_ids: matched_systems.iter().map(|sys| sys.id).collect()
                })?;
            }
        }

        if run.args.is_replay() {
            break;
        }
        
        sleep_time = match stamp == last_stamp {
            true => Duration::from_secs(2), // rest if nothing changed
            false => Duration::from_secs(0),
        };
    }
    
    if let Some(mut tls_server) = tls_server {
        io.broadcast(DataEvent::ClosingServer)?;
        let _ = tls_server.close(Duration::from_secs(20)).await;
        io.broadcast(DataEvent::ClosedServer)?;
    }
    
    if let Some(mut tls_client) = tls_client {
        io.broadcast(DataEvent::ClosingClient)?;
        let _ = tls_client.close(Duration::from_secs(20)).await;
        io.broadcast(DataEvent::ClosedClient)?;
    }
    
    io.close().await;
    Ok(())
}

async fn select_logs(run: &Running, watch_channels: &Vec<CharacterLog>, stamp: Timestamp, logs: &mut Logs) -> SolarResult<Vec<LogEntry>> {
    read_intel_logs(&run, logs)?;
    
    let activity = watch_channels.iter()
        .flat_map(|channel| logs.take_entries(channel.id))
        .filter(|entry| entry.timestamp > stamp)
        .filter(|entry| !entry.analysis.system_ids().is_empty())
        .collect::<Vec<_>>();
    
    Ok(activity)
}

async fn select_replay(run: &Running, watch_channels: &Vec<&CharacterLogIdx>, stamp: Timestamp, logs: &mut Logs, log_file: &ChatLogFile, channel_id: IndexId) -> SolarResult<Vec<LogEntry>> {
    let read = read_intel_log_file(channel_id, &log_file.path(), 0)?;
    logs.push(&run, &log_file, read);
    
    let activity = watch_channels.iter()
        .flat_map(|channel| logs.take_entries(channel.id))
        .filter(|entry| entry.timestamp > stamp)
        .filter(|entry| !entry.analysis.system_ids().is_empty())
        .collect::<Vec<_>>();
    
    Ok(activity)
}

async fn select_client(stamp: Timestamp, tls_client: &mut Option<TlsClientHandle>) -> SolarResult<Vec<LogEntry>> {
    let client = tls_client.as_mut().expect("exists");
    let events = match client.recv().await {
        Some(events) => events,
        _ => return SolarError::err_msg("Client closed"),
    };
    
    let activity = events.into_iter()
        .filter_map(|event| match event {
            DataEvent::LogEntry { entry, .. } => Some(entry),
            _ => None,
        })
        .filter(|entry| entry.timestamp > stamp)
        .filter(|entry| !entry.analysis.system_ids().is_empty())
        .collect::<Vec<_>>();
    
    Ok(activity)
}

#[derive(Debug)]
pub(crate) struct RunningParams {
    pub(crate) args: Args,
    pub(crate) arg_channel_ids: Vec<IndexId>,
    pub(crate) index: Index,
    pub(crate) cfg: Config,
    pub(crate) io: SonarOptions,
}

#[derive(Debug)]
pub(crate) struct Running {
    pub(crate) args: Args,
    pub(crate) arg_channel_ids: Vec<IndexId>,
    pub(crate) index: Index,
    pub(crate) cfg: Config,
}

pub(crate) struct Startup {
    pub(crate) running: Running,
    pub(crate) sonar_io: SonarIO,
}

impl Running {
    pub(crate) fn startup(params: RunningParams) -> SolarResult<Startup> {
        let sonar_io = SonarIO::init(params.io)?;
        
        let running = Self {
            args: params.args,
            arg_channel_ids: params.arg_channel_ids,
            index: params.index,
            cfg: params.cfg,
        };
        
        Ok(Startup { running, sonar_io })
    }
    
    pub(crate) fn watch_logs(&self) -> Vec<CharacterLog> {
        let arg_channels = self.arg_channel_ids.iter()
            .map(|id| self.index.chat_channels().get(*id).expect("exists"));
        
        self.args.watch_characters(&self.cfg).iter()
            .filter_map(|chr| {
                let logs = self.cfg.logs.iter()
                    .filter_map(|log| match log.characters.contains(&chr.id) {
                        true => Some(CharacterLog::from_kind(log.kind, chr.id, log.name)),
                        false => None,
                    })
                    .collect::<Vec<_>>();
                
                match logs.is_empty() {
                    true => None,
                    false => Some(logs),
                }
            })
            .flatten()
            //.chain(arg_channels) todo fix
            .collect()
    }
    
    pub(crate) fn watch_range(&self) -> Vec<&'static SolarSystem> {
        let nav = StarNavigator::default();
        self.args.watch_systems().iter()
            .map(|sys| nav.systems_in_range(self.args.jumps, sys))
            .flatten()
            .collect::<Vec<_>>()
    }
    
    pub(crate) fn server_profile(&self) -> Option<&ServerProfileConfig> {
        find_server_profile(&self.args, &self.cfg)
    }
    
    pub(crate) fn client_profile(&self) -> Option<&ClientProfileConfig> {
        find_client_profile(&self.args, &self.cfg)
    }
    
    pub(crate) fn index(&self) -> &Index { &self.index }
}

fn find_server_profile<'a>(args: &Args, cfg: &'a Config) -> Option<&'a ServerProfileConfig> {
    args.server_profile.as_deref().and_then(|serve_name| {
        cfg.server_profiles.iter()
            .find(|serve| serve.name == serve_name)
    })
}

fn find_client_profile<'a>(args: &Args, cfg: &'a Config) -> Option<&'a ClientProfileConfig> {
    args.client_profile.as_deref().and_then(|connect_name| {
        cfg.client_profiles.iter()
            .find(|connect| connect.name == connect_name)
    })
}


pub struct SolarSonar {
    dirs: directories::BaseDirs,
    env: SolarSonarEnv,
}

#[derive(Default)]
pub struct SolarSonarEnv {
    pub config_dir: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
}

impl SolarSonar {
    fn init(env: Option<SolarSonarEnv>) -> SolarResult<Self> {
        let dirs = directories::BaseDirs::new()
            .ok_or_else(|| SolarError::msg("Failed to load system directories"))?;
        let env = env.unwrap_or_default();

        Ok(Self{dirs, env})
    }
    
    /// XDG defaults here
    pub(crate) fn config_dir(&self) -> Cow<'_, Path> {
        match self.env.config_dir.as_ref() {
            Some(dir) => Cow::Borrowed(dir),
            None => Cow::Owned(self.dirs.home_dir().join(CONFIG_DIR)),
        }
    }
    
    pub(crate) fn data_dir(&self) -> SolarResult<Cow<'_, Path>> {
        let data_dir = match &self.env.data_dir {
            Some(dir) => Cow::Borrowed(dir.as_path()),
            None => Cow::Owned(self.dirs.data_dir().join(SOLAR_SUBDIR)),
        };
        
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)
                .map_err(|e| SolarError::mkdir(e, "Unable to make data directory"))?;
        }
        
        Ok(data_dir)
    }
    
    pub(crate) fn tilde_dir(&self) -> Cow<'_, Path> {
        Cow::Borrowed(self.dirs.home_dir())
    }

    pub fn init_once() -> SolarResult<&'static SolarSonar> {
        let runsys = SolarSonar::init(None)?;
        Ok(RUN_SYS.get_or_init(|| runsys))
    }
    
    pub fn init_once_with(env: SolarSonarEnv) -> SolarResult<&'static SolarSonar> {
        let runsys = SolarSonar::init(Some(env))?;
        Ok(RUN_SYS.get_or_init(|| runsys))
    }

    pub(crate) fn get() -> &'static Self {
        RUN_SYS.get_or_init(|| panic!("System not initialized"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceKind {
    Logs,
    Replay,
    Client,
}

pub(crate) static RUN_SYS: OnceLock<SolarSonar> = OnceLock::new();

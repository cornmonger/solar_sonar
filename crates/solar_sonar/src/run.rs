use crate::*;

pub async fn run_cli() -> ExitCode {
    if handle_error(SolarSonar::init_once()).is_err() {
        return ExitCode::FAILURE;
    }
    
    let cli = Cli::parse();
    let params = ParamsBuilder::new()
        .cli(cli)
        .build();
    let Ok(params) = handle_error(params) else {
        return ExitCode::FAILURE
    };
    
    let Ok(Startup { running, sonar_io }) = handle_error(Running::startup(params)) else {
        return ExitCode::FAILURE;
    };

    match handle_error(run(running, sonar_io).await) {
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

pub struct ParamsBuilder {
    cli: Option<Cli>,
    args: Option<Args>,
    cfg: Option<Cfg>,
}

impl ParamsBuilder {
    pub fn new() -> Self {
        Self {
            cli: None,
            args: None,
            cfg: None,
        }
    }
    
    pub fn cli(mut self, cli: Cli) -> Self {
        self.cli = Some(cli);
        self
    }
    
    pub fn args(mut self, args: Args) -> Self {
        self.args = Some(args);
        self
    }
    
    pub fn cfg(mut self, cfg: Cfg) -> Self {
        self.cfg = Some(cfg);
        self
    }
    
    pub fn build(self) -> SolarResult<Params> {
        let is_cli = self.cli.is_some();
        match (is_cli, self.args.is_some(), self.cfg.is_some()) {
            (true, false, false) | (false, true, true) => Ok(()),
            (true, true, true) | (true, false, true) | (true, true, false) =>
                SolarError::err_msg("Cli is mutually exclusive with Args & Cfg"),
            (false, false, true) => SolarError::err_msg("Args is required"),
            (false, true, false) => SolarError::err_msg("Cfg is required"),
            (false, false, false) =>
                SolarError::err_msg("Either Cli or Args & Cfg is required"),
        }?;
        
        SolarSonar::init_once()?;
        
        let (cfg_param, args_param) = match is_cli {
            true => Self::build_cli(self.cli.expect("some")),
            false => Self::build_args(self.cfg.expect("some"), self.args.expect("some")),
        }?;
        
        
        let CfgParam { config, index } = cfg_param;
        let ArgParam { args, arg_indexed_logs, mode_id } = args_param;
        
        check_args(&args, &config)?;
        
        let opts = SonarOptions {
            audio: args.audio,
            stdio: args.stdio,
            binlog: args.binlog.clone(),
        };
        
        Ok(Params {
            index,
            cfg: config,
            args,
            arg_indexed_logs,
            opts,
            mode_id,
        })
    }
    
    fn build_cli(cli: Cli) -> SolarResult<(CfgParam, ArgParam)> {
        let mut cfg_param = Cfg::read()?.build()?;
        let arg_param = Args::try_from_cli(cli, &cfg_param.config, &mut cfg_param.index)?;
        Ok((cfg_param, arg_param))
    }
    
    fn build_args(cfg: Cfg, args: Args) -> SolarResult<(CfgParam, ArgParam)> {
        let mut cfg_param = cfg.build()?;
        let arg_param = args.build(&mut cfg_param.index)?;
        Ok((cfg_param, arg_param))
    }
}

pub struct Params {
    pub(crate) cfg: Config,
    pub(crate) index: Index,
    pub(crate) args: Args,
    pub(crate) arg_indexed_logs: Vec<IndexedLog>,
    pub(crate) opts: SonarOptions,
    pub(crate) mode_id: ModeId,
}

pub fn start(params: Params) -> SolarResult<SolarSonarHandle> {
    let Startup { running, sonar_io } = handle_error(Running::startup(params))?;
    let rx = sonar_io.subscribe();
    
    let cancel = r::tokio::CancellationToken::new();
    let handle = tokio::task::spawn(async move {
        handle_error(run(running, sonar_io).await)
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

async fn run(run: Running, mut io: SonarIO) -> SolarResult<()> {
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
    } else if run.args.replay_path.is_some() {
        SourceKind::Replay
    } else {
        SourceKind::Logs
    };
    
    let (mut last_stamp, replay_log) = match &run.args.replay_path {
        Some(p) => {
            let log_dir_kind = LogDirKind::from_file_path(p)
                .ok_or_else(|| SolarError::msg(format!("Unable to determine log kind from dirname: {}", log_path(p))))?;
            let log = LogFile::from_path_buf(p.to_path_buf(), log_dir_kind)
                .ok_or_else(|| SolarError::msg(format!("Invalid chat log: {}", log_path(p))))?;

            (log.timestamp().clone(), Some(log))
        },
        None => (Utc::now().into(), None),
    };

    io.broadcast(DataEvent::PingFortune)?;

    let mut logs = Logs::new_watch(&run);
    let mut analyzer_state = AnalyzerState::new();
    let mut sleep_time = Duration::from_secs(0);
    let signal_ctl_c = tokio::signal::ctrl_c();
    tokio::pin!(signal_ctl_c);
    
    loop {
        let stamp = last_stamp;
        
        let source: BoxFuture<Option<SolarResult<Vec<LogEntry>>>> = match source_kind {
            SourceKind::Logs => {
                let run = &run;
                let watch_channels = &watch_channels;
                let logs = &mut logs;
                let analyzer_state = &mut analyzer_state;
                Box::pin(async move { select_logs(&run, &watch_channels, stamp, analyzer_state, logs).await })
            },
            SourceKind::Replay => {
                let run = &run;
                let replay_log = replay_log.as_ref().expect("exists");
                let logs = &mut logs;
                let analyzer_state = &mut analyzer_state;
                Box::pin(async move { select_replay(&run, stamp, analyzer_state, logs, &replay_log).await })
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
            Some(result) = source => match result {
                Ok(activity) => activity,
                Err(_) => break,
            },
            _ = tokio::time::sleep(sleep_time) => vec![],
        };
        for entry in activity {
            dbg!(&entry);
            last_stamp = entry.timestamp;
            let matched_systems = entry.analysis.system_ids().iter()
                .filter_map(|sys_id| watch_range.iter().find(|sys| &sys.id == sys_id))
                .collect::<Vec<_>>();

            let in_range = !matched_systems.is_empty();
            let dangerous = entry.analysis.dangerous();
            let dangerous_callout = entry.analysis.dangerous_callout();
            let in_danger = dangerous && (in_range || dangerous_callout);
            let channel = entry.character_log; // todo: copy only when needed
            
            io.broadcast(DataEvent::LogEntry { entry, in_range, in_danger, dangerous })?;

            if in_danger {
                if in_range {
                    io.broadcast(DataEvent::PingSystems {
                        system_ids: matched_systems.iter().map(|sys| sys.id).collect()
                    })?;
                } else if dangerous_callout {
                    io.broadcast(DataEvent::PingChannel {
                        channel, 
                    })?;
                } 
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

async fn select_logs(run: &Running, watch_logs: &Vec<CharacterLog>, stamp: Timestamp, analyzer_state: &mut AnalyzerState, logs: &mut Logs) -> Option<SolarResult<Vec<LogEntry>>> {
    let result = (|| {
        read_logs(&run, watch_logs, analyzer_state, logs)?;
        
        let activity = watch_logs.iter()
            .flat_map(|channel| logs.take_entries(channel))
            .filter(|entry| entry.timestamp > stamp)
            //.filter(|entry| !entry.analysis.system_ids().is_empty())
            .collect::<Vec<_>>();
            
        match activity.is_empty() {
            true => Ok(None),
            false => Ok(Some(activity)),
        }
    })();
    
    result.transpose()
}

async fn select_replay(run: &Running, stamp: Timestamp, analyzer_state: &mut AnalyzerState, logs: &mut Logs, log_file: &LogFile) -> Option<SolarResult<Vec<LogEntry>>> {
    let result = (|| {
        let character_log = log_file.to_character_log(run.index())?;
        let analyze = &run.cfg.find_character_log(&character_log)?.analysis;
        let mode_cfg = run.mode_config();
        let read = read_log_file(character_log, analyze, &mode_cfg, &log_file.path(), analyzer_state, 0)?;
        logs.push(&log_file, read);
        
        let activity = logs.take_entries(&character_log).into_iter()
            .filter(|entry| entry.timestamp > stamp)
            .filter(|entry| !entry.analysis.system_ids().is_empty())
            .collect::<Vec<_>>();
        
        match activity.is_empty() {
            true => Ok(None),
            false => Ok(Some(activity)),
        }
    })();
    
    result.transpose()
}

async fn select_client(stamp: Timestamp, tls_client: &mut Option<TlsClientHandle>) -> Option<SolarResult<Vec<LogEntry>>> {
    let result = (async || {
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
        
        match activity.is_empty() {
            true => Ok(None),
            false => Ok(Some(activity)),
        }
    })().await;
    
    result.transpose()
}

#[derive(Debug)]
pub(crate) struct Running {
    pub(crate) args: Args,
    pub(crate) arg_indexed_logs: Vec<IndexedLog>,
    pub(crate) index: Index,
    pub(crate) cfg: Config,
    pub(crate) mode_id: ModeId,
}

pub(crate) struct Startup {
    pub(crate) running: Running,
    pub(crate) sonar_io: SonarIO,
}

impl Running {
    pub(crate) fn startup(params: Params) -> SolarResult<Startup> {
        let sonar_io = SonarIO::init(params.opts)?;
        
        let running = Self {
            args: params.args,
            arg_indexed_logs: params.arg_indexed_logs,
            index: params.index,
            cfg: params.cfg,
            mode_id: params.mode_id,
        };
        
        Ok(Startup { running, sonar_io })
    }
    
    pub(crate) fn watch_logs(&self) -> Vec<CharacterLog> {
        let watch_char_ids = self.args.watch_characters(&self.cfg).iter()
            .map(|chr| chr.id)
            .collect::<Vec<_>>();
        
        let logs_arg = self.arg_indexed_logs.iter()
            .map(|indexed| watch_char_ids.iter()
                .map(|char_id| CharacterLog::from_indexed(*indexed, *char_id))
            )
            .flatten();
        
        let watch = self.cfg.logs.iter()
            .filter_map(|log_cfg| {
                let char_logs = watch_char_ids.iter()
                    .filter_map(|char_id| match log_cfg.characters.contains(&char_id) {
                        true => Some(CharacterLog::from_indexed(log_cfg.indexed, *char_id)),
                        false => None,
                    })
                    .collect::<Vec<_>>();
                
                match char_logs.is_empty() {
                    true => None,
                    false => Some(char_logs),
                }
            })
            .flatten()
            .chain(logs_arg)
            .dedup()
            .collect::<Vec<_>>();
        
        watch
    }
    
    pub(crate) fn watch_range(&self) -> Vec<&'static StarSystem> {
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
    
    pub(crate) fn mode_config(&self) -> &ModeConfig {
        self.cfg.modes.iter()
            .find(|mode_cfg| mode_cfg.id == self.mode_id)
            .expect("mode config exists")
    }
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

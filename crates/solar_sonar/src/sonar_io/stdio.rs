use crate::*;

#[derive(Debug)]
pub(crate) struct SonarStdio;

impl SonarStdio {
    pub(crate) fn new() -> Self { Self }
}

impl SonarListener for SonarStdio {
    async fn on_cancel(&self, _run: &Running) {}
    
    async fn on_event(&self, run: &Running, event: &DataEvent) -> SolarResult<()> {
        const DONE: &'static str = "done";
        const FAILED: &'static str = "failed";
        
        match event {
            DataEvent::Args(data) => on_args(run, data)?,
            DataEvent::ClosingClient => { 
                print!("{INFO} closing server connections ... ");
                let _ = io::stdout().flush();
            },
            DataEvent::ClosedClient => {
                fin(&true, DONE, FAILED);
            },
            DataEvent::ClosingServer => {
                print!("{INFO} closing server connections ... ");
                let _ = io::stdout().flush();
            },
            DataEvent::ClosedServer => {
                fin(&true, DONE, FAILED);
            },
            DataEvent::Downloading { id } => {
                let name = RemoteAssets::get(*id)?.name;
                print!("{INFO} downloading asset {MAGENTA}{name}{CLR} ... ");
                let _ = io::stdout().flush();
            },
            DataEvent::Download { id: _, success } => {
                fin(success, DONE, FAILED);
            },
            DataEvent::GeneratingCerts => {
                print!("{INFO} generating TLS certificates ... ");
                let _ = io::stdout().flush();
            },
            DataEvent::GenerateCerts { success } => {
                fin(success, DONE, FAILED);
                if *success {
                    println!("{INFO} please distribute to clients: {MAGENTA}~/.config/solar_sonar/certs/authority_server.pem{CLR}");
                }
            },
            DataEvent::LogEntry { entry, in_range, in_danger } => {
                println!("{}", entry.display_ansi(*in_range, *in_danger));
            },
            DataEvent::PingFortune => {},
            DataEvent::PingSystems { .. } => {},
            DataEvent::Connecting { to } => {
                println!("{INFO} connecting to {MAGENTA}{to}{CLR}");
            },
            DataEvent::Connect { to, success } => {
                match success {
                    true => println!("{INFO} connected to {MAGENTA}{to}{CLR}"),
                    false => println!("{ERR} failed to connect to {MAGENTA}{to}{CLR}"),
                }
            },
        }
        
        Ok(())
    }
}

fn fin(success: &bool, pass: &'static str, fail: &'static str) {
    match success {
        true => println!("{GREEN}{pass}{CLR}"),
        false => println!("{RED}{fail}{CLR}"),
    }
}

fn on_args(run: &Running, data: &ArgsData) -> SolarResult<()> {
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
    
    let channels = run.watch_logs().into_iter()
        .map(|s| format!("{GREEN}{s}{CLR}"))
        .collect::<Vec<_>>()
        .join(" ");
    
    let range = run.watch_range().iter()
        .map(|sys| format!("{BRIGHT_BLUE}{}{CLR}", sys.name))
        .collect::<Vec<_>>()
        .join(" ");
    
    println!("{INFO} watching {systems} for {characters} in {channels}");
    println!("{INFO} within {} jumps: {range}", data.jumps);
    
    Ok(())
}
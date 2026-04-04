use crate::*;

#[derive(Debug, Clone, clap::Parser)]
pub struct Cli {
    pub mode: String,
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
    /// Enable / disable audio alerts
    #[clap(long, default_value_t = Args::DEFAULT_AUDIO, action = clap::ArgAction::Set)]
    pub audio: bool,
    /// Replay a specific log file
    #[clap(long)]
    pub replay: Option<PathBuf>,
    #[clap(long)]
    /// Writes events to a binary log directory
    pub relog: Option<PathBuf>,
    /// Serve events to clients according to bind.toml profile.
    #[clap(long)]
    pub serve: Option<String>,
    /// Receive events from a connect.toml profile.
    #[clap(long)]
    pub connect: Option<String>,
}

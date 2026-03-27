use crate::*;

#[derive(
    Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub enum DataEvent {
    Args(ArgsData), 
    ClosingServer,
    ClosedServer,
    ClosingClient,
    ClosedClient,
    Connecting {
        to: String,
    },
    Connect {
        to: String,
        success: bool,
    },
    Downloading {
        id: u8,
    },
    Download {
        id: u8,
        success: bool,
    },
    GeneratingCerts,
    GenerateCerts { success: bool },
    LogEntry {
        entry: LogEntry,
        in_range: bool,
        in_danger: bool,
        dangerous: bool,
    },
    PingFortune,
    PingChannel {
        channel: CharacterLog,
    },
    PingSystems {
        system_ids: Vec<SolarId>,
    },
}

#[derive(
    Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub struct ArgsData {
    pub system_ids: Vec<SolarId>,
    pub character_ids: Vec<CharacterId>,
    pub channels: Vec<CharacterLog>,
    pub jumps: u8,
    pub system_range: Vec<SolarId>,
}

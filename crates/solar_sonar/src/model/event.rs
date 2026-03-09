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
    },
    PingFortune,
    PingSystems {
        system_ids: Vec<SolarID>,
    },
}

#[derive(
    Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub struct ArgsData {
    pub system_ids: Vec<SolarID>,
    pub character_ids: Vec<CharacterID>,
    pub channels: Vec<String>,
    pub jumps: u8,
    pub system_range: Vec<SolarID>,
}

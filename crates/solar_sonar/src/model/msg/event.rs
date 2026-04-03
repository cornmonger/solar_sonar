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
        system_ids: Vec<StarId>,
    },
}

#[derive(
    Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub struct ArgsData {
    pub system_ids: Vec<StarId>,
    pub character_ids: Vec<CharacterId>,
    pub channels: Vec<CharacterLog>,
    pub jumps: u8,
    pub system_range: Vec<StarId>,
}


#[derive(
    Debug, Clone, PartialEq,
    serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub struct DataLog {
    pub events: Vec<DataEvent>,
}

impl DataLog {
    pub fn new(events: Vec<DataEvent>) -> Self { Self { events } }
    
    pub fn read_ron(filepath: &Path) -> SolarResult<Self> {
        let ron_options = make_ron_options();
        let txt = fs::read_to_string(filepath)
            .map_err(|e| SolarError::read(e, filepath))?;
        ron_options.from_str::<Vec<DataEvent>>(&txt)
            .map(|events| Self { events })
            .map_err(|e| SolarError::msg(e.to_string()))
    }
    
    pub fn write_ron(&self, filepath: &Path) -> SolarResult<()> {
        let ron_options = make_ron_options();
        let ron_config = make_ron_config(&ron_options);
        let file = fs::File::options()
            .write(true)
            .open(filepath)
            .map_err(|e| SolarError::write(e, filepath))?;
        let mut writer = BufWriter::new(file);
        
        writer.write_all("[\n".as_bytes())
            .map_err(|e| SolarError::write(e, filepath))?;
        
        for event in &self.events {
            let ron_cfg = ron_config.clone();
            let txt = ron_options.to_string_pretty(event, ron_cfg)
                .map_err(|e| SolarError::msg(e.to_string()))?;
            let txt = txt + ",\n";
            
            writer.write_all(txt.as_bytes())
                .map_err(|e| SolarError::write(e, filepath))?;
        }
        
        writer.write_all("]\n".as_bytes())
            .map_err(|e| SolarError::write(e, filepath))?;
        
        Ok(())
    }
    
    pub fn append_ron(events: &Vec<DataEvent>, filepath: &Path) -> SolarResult<()> {
        let ron_options = make_ron_options();
        let ron_config = make_ron_config(&ron_options);
        
        let mut file = File::options()
            .append(true)
            .open(filepath)
            .map_err(|e| SolarError::write(e, filepath))?;
        file.seek(SeekFrom::End(-1))
            .map_err(|e| SolarError::write(e, filepath))?;
        let mut writer = BufWriter::new(file);
        
        for event in events {
            let ron_cfg = ron_config.clone();
            let txt = ron_options.to_string_pretty(event, ron_cfg)
                .map_err(|e| SolarError::msg(e.to_string()))?;
            let txt = txt + ",\n";
            
            writer.write_all(txt.as_bytes())
                .map_err(|e| SolarError::write(e, filepath))?;
        }
        
        writer.write_all("]".as_bytes())
            .map_err(|e| SolarError::write(e, filepath))?;
        
        Ok(())
    }
}

fn make_ron_options() -> ron::Options {
    ron::Options::default().with_default_extension(
        ron::extensions::Extensions::UNWRAP_NEWTYPES
        | ron::extensions::Extensions::UNWRAP_VARIANT_NEWTYPES
        | ron::extensions::Extensions::IMPLICIT_SOME
        | ron::extensions::Extensions::EXPLICIT_STRUCT_NAMES
    )
}

fn make_ron_config(ron_options: &ron::Options) -> ron::ser::PrettyConfig {
    ron::ser::PrettyConfig::default()
        .struct_names(true)
        .extensions(ron_options.default_extensions)
}

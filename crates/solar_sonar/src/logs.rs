use chrono::TimeZone;
use std::str::FromStr;

use crate::*;

#[derive(Debug)]
pub(crate) struct CharacterLogFile {
    pub(crate) character_log: CharacterLog,
    pub(crate) file: LogFile,
}

#[derive(Debug)]
pub(crate) struct LogSource {
    file: PathBuf,
    cursor: u64,
}

#[derive(Debug)]
pub(crate) struct Log {
    pub character_log: CharacterLog,
    pub entries: Vec<LogEntry>,
    sources: HashMap<CharacterId, LogSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
    serde::Serialize, serde::Deserialize, bitcode::Encode, bitcode::Decode,
)]
pub struct Timestamp(pub i64, pub u32);

impl PartialEq<DateTime<Utc>> for Timestamp {
    fn eq(&self, other: &DateTime<Utc>) -> bool {
        self.0 == other.timestamp() && self.1 == other.timestamp_subsec_nanos()
    }
}

impl PartialOrd<DateTime<Utc>> for Timestamp {
    fn partial_cmp(&self, other: &DateTime<Utc>) -> Option<std::cmp::Ordering> {
        match (self.0.cmp(&other.timestamp()), self.1.cmp(&other.timestamp_subsec_nanos())) {
            (std::cmp::Ordering::Equal, ord) => Some(ord),
            (ord, std::cmp::Ordering::Equal) => Some(ord),
            (ord, _) => Some(ord),
        }
    }
}

impl Timestamp {
    pub fn to_datetime(&self) -> DateTime<Utc> {
        Utc.timestamp_opt(self.0, self.1).single().expect("valid timestamp")
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(value: DateTime<Utc>) -> Self {
        Timestamp(value.timestamp(), value.timestamp_subsec_nanos())
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(value: Timestamp) -> Self { value.to_datetime() }
}

#[derive(
    Debug, Clone, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub struct LogEntry {
    pub character_log: CharacterLog,
    pub timestamp: Timestamp,
    pub author: ChatAuthor,
    pub content: String,
    pub analysis: LogAnalysis,
}

#[derive(
    Debug, Clone, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub enum ChatAuthor {
    System,
    Character(String),
}

impl ChatAuthor {
    const SYSTEM: &'static str = "System";
    
    pub fn as_str(&self) -> &str {
        match self {
            Self::System => Self::SYSTEM,
            Self::Character(name) => name.as_str(),
        }
    }
}

impl AsRef<str> for ChatAuthor { fn as_ref(&self) -> &str { self.as_str() } }

#[derive(Debug)]
pub(crate) struct Logs(HashMap<CharacterLog, Log>);

impl Logs {
    pub fn new_watch(run: &Running) -> Self {
        let logs = run.watch_logs().into_iter() 
            .map(|character_log| Log { character_log, entries: vec![], sources: HashMap::new() })
            .fold(HashMap::new(), |mut map, log| { map.insert(log.character_log, log); map });

        Self(logs)
    }

    pub fn get_mut(&mut self, character_log: CharacterLog) -> Option<&mut Log> {
        self.0.get_mut(&character_log)
    }

    pub(crate) fn push(&mut self, logfile: &LogFile, read: LogRead) {
        //let channel_id = run.index().chat_channels().find(logfile.name()).expect("chan");
        if self.0.contains_key(&read.character_log) {
            let log = self.0.get_mut(&read.character_log).expect("exists");
            log.entries.extend(read.entries);
            log.sources
                .entry(logfile.character_id())
                .and_modify(|src| {
                    let path = logfile.path();
                    if src.file != path {
                        src.file = path.to_path_buf();
                        src.cursor = 0;
                    }
                })
                .or_insert(LogSource { file: logfile.path().to_path_buf(), cursor: read.cursor });

        } else {
            let log = Log {
                character_log: read.character_log,
                entries: read.entries,
                sources: HashMap::from([
                    (
                        logfile.character_id(),
                        LogSource {
                            file: logfile.path().to_path_buf(),
                            cursor: read.cursor,
                        }
                    ),
                ]),
            };

            self.0.insert(read.character_log, log);
        }
    }
    
    pub(crate) fn take_entries(&mut self, character_log: &CharacterLog) -> Vec<LogEntry> {
        self.0.get_mut(&character_log).map(|log| {
            std::mem::take(&mut log.entries)
        }).unwrap_or_default()
    }
}

#[ouroboros::self_referencing]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LogFile {
    pub(crate) path: PathBuf,
    #[borrows(path)]
    #[covariant]
    parts: ChatLogFileParts<'this>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ChatLogFileParts<'a> {
    pub(crate) name: LogName<'a>,
    pub(crate) timestamp: Timestamp,
    pub(crate) character_id: CharacterId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LogName<'a> {
    Game,
    Chat(&'a str),
}

impl<'a> LogName<'a> {
    pub(crate) fn channel_name(&self) -> Option<&'a str> {
        match self {
            Self::Game => None,
            Self::Chat(name) => Some(name),
        }
    }
}

impl LogFile {
    pub fn path(&self) -> &Path {
        &self.borrow_path()
    }

    pub fn name(&self) -> &LogName<'_> {
        &self.borrow_parts().name
    }

    pub fn character_id(&self) -> CharacterId {
        self.borrow_parts().character_id
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.borrow_parts().timestamp
    }

    pub fn from_path_buf(path: PathBuf, dir_kind: LogDirKind) -> Option<Self> {
        const TXT: &'static str = "txt";
        const UNDERSCORE: char = '_';

        // example: "/path/to/our.intel_20260212_035141_12345678.txt";

        Self::try_new(path, |path| {
            if Some(TXT) != path.extension().and_then(|p| p.to_str()) {
                return Err(())
            }
            let Some(file) = path.file_stem().and_then(|p| p.to_str()) else {
                return Err(())
            };
            let Some((file, character_id)) = file.rsplit_once(UNDERSCORE) else {
                return Err(())
            };
            let Some((file, time)) = file.rsplit_once(UNDERSCORE) else {
                return Err(())
            };
            let (name, date) = match dir_kind {
                LogDirKind::Game => {
                    (LogName::Game, file)
                },
                LogDirKind::Chat => {
                    let Some((name, date)) = file.rsplit_once(UNDERSCORE) else {
                        return Err(())
                    };
                    
                    (LogName::Chat(name), date)
                },
            };
            let Ok(character_id) = CharacterId::from_str(character_id) else {
                return Err(())
            };
            let Some(datetime) = make_datetime(date, time) else {
                return Err(())
            };
            let timestamp = Timestamp::from(datetime);

            Ok(ChatLogFileParts {
                name,
                timestamp,
                character_id,
            })
        })
        .ok()
    }
    
    pub(crate) fn to_character_log(&self, index: &Index) -> SolarResult<CharacterLog> {
        let log_name = self.name();
        let character_id = self.character_id();
        let log_kind = LogKind::from_log_file(&self);
        let channel_name_id = match log_kind {
            LogKind::Group => {
                let channel_name = log_name.channel_name()
                    .ok_or_else(|| SolarError::not_found(ErrNoun::ChannelName, self.path().to_string_lossy()))?;
                let channel_name_id = index.chat_channels().find(channel_name)?.id();
                Some(channel_name_id)
            }
            _ => None,
        };
        
        Ok(CharacterLog::from_kind(log_kind, character_id, channel_name_id))
    }
}

fn make_datetime(date: &str, time: &str) -> Option<DateTime<Utc>> {
    let Some(Ok(year)) = date.get(0..4).map(i32::from_str) else { return None };
    let Some(Ok(month)) = date.get(4..6).map(u32::from_str) else { return None };
    let Some(Ok(day)) = date.get(6..8).map(u32::from_str) else { return None };
    let Some(Ok(hour)) = time.get(0..2).map(u32::from_str) else { return None };
    let Some(Ok(minute)) = time.get(2..4).map(u32::from_str) else { return None };
    let Some(Ok(second)) = time.get(4..6).map(u32::from_str) else { return None };

    Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LogDirKind {
    Game,
    Chat,
}

impl LogDirKind {
    const GAME_LOGS: &'static str = "Gamelogs";
    const CHAT_LOGS: &'static str = "Chatlogs";
    
    pub(crate) fn dir_name(&self) -> &'static str {
        match self {
            Self::Game => Self::GAME_LOGS,
            Self::Chat => Self::CHAT_LOGS,
        }
    }
    
    pub(crate) fn from_file_path(path: &Path) -> Option<Self> {
        match path.parent() {
            Some(dir) => match dir.file_name().and_then(|s| s.to_str()) {
                Some(dir_name) => Self::from_dir_name(dir_name),
                None => None,
            },
            None => None,
        }
    }
    
    pub(crate) fn from_dir_name(dir_name: &str) -> Option<Self> {
        match dir_name {
            Self::GAME_LOGS => Some(Self::Game),
            Self::CHAT_LOGS => Some(Self::Chat),
            _ => None,
        }
    }
}

pub(crate) fn read_logs(run: &Running, watch_logs: &Vec<CharacterLog>, logs: &mut Logs) -> SolarResult<()> {
    read_log_dir(run, watch_logs, logs, LogDirKind::Game)?;
    read_log_dir(run, watch_logs, logs, LogDirKind::Chat)?;
    Ok(())
}

pub(crate) fn read_log_dir(run: &Running, watch_logs: &Vec<CharacterLog>, logs: &mut Logs, dir_kind: LogDirKind) -> SolarResult<()> {
    let chat_logs_dir = run.cfg.logs_dir.join(dir_kind.dir_name());
    let index = run.index();

    let log_files = fs::read_dir(&chat_logs_dir)
        .map_err(|e| SolarError::list(e, &chat_logs_dir))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let Some(file) = LogFile::from_path_buf(path, dir_kind) else {
                return None
            };

            let Ok(character_log) = file.to_character_log(&index) else {
                return None
            };
            if !watch_logs.contains(&character_log) {
                return None
            }

            Some(CharacterLogFile { file, character_log })
        })
        .into_group_map_by(|log| log.character_log)
        .into_iter()
        .map(|(_k, mut v)| {
            v.sort_by(|a, b| a.file.timestamp().cmp(b.file.timestamp()));
            v.pop().expect("value")
        })
        .collect::<Vec<_>>();
    
    for log_file in log_files {
        let log = logs.get_mut(log_file.character_log).expect("log exists");
        let log_source = {
            let source = log.sources
                .entry(log_file.character_log.character_id())
                .or_insert(LogSource { file: log_file.file.path().to_path_buf(), cursor: 0 });

            if source.file != log_file.file.path() {
                source.file = log_file.file.path().to_path_buf();
                source.cursor = 0;
            }

            source
        };

        let analyze = &run.cfg.find_character_log(&log_file.character_log)?.analysis;
        let read = read_log_file(log_file.character_log, &analyze, log_file.file.path(), log_source.cursor)?;
        log_source.cursor = read.cursor;
        log.entries.extend(read.entries);
    }

    Ok(())
}

#[derive(Debug)]
pub(crate) struct LogRead {
    pub(crate) character_log: CharacterLog,
    pub(crate) cursor: u64,
    pub(crate) entries: Vec<LogEntry>,
}

pub(crate) fn read_log_file(character_log: CharacterLog, analyze: &Vec<AnalysisKind>, filepath: &Path, mut cursor: u64) -> SolarResult<LogRead> {
    let is_utf8 = !character_log.is_chat();
    let mut entries: Vec<LogEntry> = vec![];

    let mut file = File::open(&filepath)
        .map_err(|e| SolarError::read(e, &filepath))?;
    file.seek(SeekFrom::Start(cursor))
        .map_err(|e| SolarError::read(e, &filepath))?;
    
    let encoding = match is_utf8 {
        false => Some(UTF_16LE),
        true => None,
    };
    
    let file = DecodeReaderBytesBuilder::new()
        .encoding(encoding)
        .build(file);
    let mut reader = BufReader::new(file);

    loop {
        let mut buf = String::new();
        let size = reader.read_line(&mut buf)
            .map_err(|e| SolarError::read(e, &filepath))?;

        if size == 0 || buf.chars().last() != Some('\n') { break }
        let transcode_size = match is_utf8 {
            false => buf.encode_utf16().count() * 2,
            true => buf.len(),
        };
        cursor += transcode_size as u64;

        let line = buf.trim().trim_start_matches('\u{feff}');
        if !line.starts_with('[') { continue }
        let pos = line.char_indices().nth(24).map(|(i,_)| i).unwrap_or(line.len());
        let Ok(stamp) = NaiveDateTime::parse_from_str(&line[..pos], "[ %Y.%m.%d %H:%M:%S ] ") else {
            continue
        };
        let datetime = stamp.and_utc();
        let line = &line[pos..];
        
        let (author, content) = match character_log.is_chat() {
            true => {
                let Some(pos) = line.find(" > ") else { continue };
                let author = (&line[..pos]).to_string();
                let pos = line.char_indices().nth(pos + 3).map(|(i,_)| i).unwrap_or(line.len());
                let content = (&line[pos..]).to_string();
                let author = ChatAuthor::Character(author);
                (author, content)
            }
            false => {
                let content = (&line[..]).to_string();
                (ChatAuthor::System, content)
            },
        };

        
        let intel = if analyze.contains(&AnalysisKind::Intel) {
            Some(IntelAnalyzer.analyze(&content))
        } else { None };
        let callout = if analyze.contains(&AnalysisKind::Callout) {
            Some(CalloutAnalyzer.analyze(&content))
        } else { None };
        
        let analysis = LogAnalysis {
            intel,
            callout,
        };

        let timestamp = datetime.into();

        let entry = LogEntry {
            character_log,
            timestamp,
            author,
            content,
            analysis,
        };

        entries.push(entry);
    }

    Ok(LogRead { character_log, cursor, entries })
}

impl LogEntry {
    pub fn display_ansi<'this,'run:'this>(&'this self, index: &'run Index, in_range: bool, in_danger: bool, dangerous: bool) -> LogEntryAnsi<'this,'run> {
        LogEntryAnsi {
            entry: self,
            index,
            in_range,
            in_danger,
            dangerous,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_log_filename() {
        const FILENAME: &'static str = "our.intel_20260212_035141_12345678.txt";
        let expected = Some(LogFile::new(PathBuf::from(FILENAME), |_| {
            let timestamp = Utc.with_ymd_and_hms(2026, 2, 12, 3, 51, 41)
                .single().unwrap()
                .into();
            ChatLogFileParts {
                name: LogName::Chat("our.intel"),
                character_id: 12345678,
                timestamp, 
            }
        }));

        let actual = LogFile::from_path_buf(PathBuf::from(FILENAME), LogDirKind::Chat);
        assert_eq!(expected, actual);
    }
}

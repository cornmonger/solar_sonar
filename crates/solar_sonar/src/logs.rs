use chrono::TimeZone;
use std::str::FromStr;

use crate::*;

#[derive(Debug)]
pub(crate) struct ConfiguredChatLog<'a> {
    pub(crate) file: ChatLogFile,
    pub(crate) character_log: CharacterLog,
    pub(crate) character_cfg: &'a CharacterConfig,
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

#[derive(
    Debug, Clone, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub enum LogAnalysis {
    Intel(IntelLogAnalysis),
}

impl LogAnalysis {
    pub fn system_ids(&self) -> &Vec<SolarId> {
        match self {
            Self::Intel(info) => &info.systems,
        }
    }
    
    pub fn in_danger(&self) -> bool {
        match self {
            Self::Intel(info) => info.in_danger(),
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub struct IntelLogAnalysis {
    pub ambiguous: bool,
    pub systems: Vec<SolarId>,
    pub keywords: Vec<LogKeyword>,
}

impl IntelLogAnalysis {
    pub fn in_danger(&self) -> bool {
        if self.systems.is_empty() {
            return false;
        }

        self.keywords.iter()
            .fold(None, |danger, word| match danger {
                None => Some(word.danger(self.ambiguous)),
                Some(last) => Some(last || word.danger(self.ambiguous)),
            })
            .unwrap_or_else(|| true)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub enum LogKeyword {
    Clear,
    NoVisual,
    Status,
}

impl LogKeyword {
    pub fn matches(s: &str) -> Option<Self> {
        match s {
            "CLEAR" | "CLR" => Some(Self::Clear),
            "NV" => Some(Self::NoVisual),
            "STATUS" | "STATUS?" | "CLR?" | "CLEAR?" => Some(Self::Status),
            _ => None
        }
    }

    pub fn danger(&self, has_unknown: bool) -> bool {
        match (self, has_unknown) {
            (Self::Clear, false) => false,
            (Self::Clear, true) => true,
            (Self::NoVisual, _) => true,
            (Self::Status, false) => false,
            (Self::Status, true) => true,
        }
    }
}

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

    pub(crate) fn push(&mut self, run: &Running, logfile: &ChatLogFile, read: LogRead) {
        let channel_id = run.index().chat_channels().find(logfile.channel()).expect("chan");
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
pub(crate) struct ChatLogFile {
    pub(crate) path: PathBuf,
    #[borrows(path)]
    #[covariant]
    parts: ChatLogFileParts<'this>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ChatLogFileParts<'a> {
    pub(crate) channel: &'a str,
    pub(crate) timestamp: Timestamp,
    pub(crate) character_id: CharacterId,
}

impl ChatLogFile {
    pub fn path(&self) -> &Path {
        &self.borrow_path()
    }

    pub fn channel(&self) -> &str {
        self.borrow_parts().channel
    }

    pub fn character_id(&self) -> CharacterId {
        self.borrow_parts().character_id
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.borrow_parts().timestamp
    }

    pub fn from_path_buf(path: PathBuf) -> Option<Self> {
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
            let Some((channel, date)) = file.rsplit_once(UNDERSCORE) else {
                return Err(())
            };
            let Ok(character_id) = CharacterId::from_str(character_id) else {
                return Err(())
            };
            let Some(datetime) = make_datetime(date, time) else {
                return Err(())
            };
            let timestamp = Timestamp::from(datetime);

            Ok(ChatLogFileParts {
                channel,
                timestamp,
                character_id,
            })
        })
        .ok()
    }
    
    pub(crate) fn to_character_log(&self, index: &Index) -> SolarResult<CharacterLog> {
        let channel_name = self.channel();
        let character_id = self.character_id();
        let log_kind = LogKind::from_log_name(channel_name);
        let channel_name_id = match log_kind {
            LogKind::Group => Some(index.chat_channels().find(channel_name)?.id()),
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

pub(crate) fn read_intel_logs(run: &Running, logs: &mut Logs) -> SolarResult<()> {
    let chat_logs_dir = run.cfg.logs_dir.join("Chatlogs");
    let watch_chrs = run.args.watch_characters(&run.cfg);
    let index = run.index();

    let chatlogs = fs::read_dir(&chat_logs_dir)
        .map_err(|e| SolarError::list(e, &chat_logs_dir))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let Some(file) = ChatLogFile::from_path_buf(path) else {
                return None
            };

            let character_id = file.character_id();
            let Ok(character_log) = file.to_character_log(&index) else {
                return None
            };
            let log_kind = character_log.to_kind();
            let channel_name_id = character_log.channel_name_id();
            let Some(character_cfg) = watch_chrs.iter().find(|chr| chr.id == character_id) else {
                return None
            };

            let log_cfg = run.cfg.logs.iter()
                .find(|log| {
                    log.kind == log_kind
                    && log.name == channel_name_id
                    && log.characters.contains(&character_id)
                });
            let Some(log_cfg) = log_cfg else {
                return None
            };
            
            Some(ConfiguredChatLog { file, character_cfg, character_log })
        })
        .into_group_map_by(|log| log.character_log)
        .into_iter()
        .map(|(_k, mut v)| {
            v.sort_by(|a, b| a.file.timestamp().cmp(b.file.timestamp()));
            v.pop().expect("value")
        })
        .collect::<Vec<_>>();

    for chatlog in chatlogs {
        let log = logs.get_mut(chatlog.character_log).expect("log exists");
        let log_source = {
            let source = log.sources
                .entry(chatlog.character_cfg.id)
                .or_insert(LogSource { file: chatlog.file.path().to_path_buf(), cursor: 0 });

            if source.file != chatlog.file.path() {
                source.file = chatlog.file.path().to_path_buf();
                source.cursor = 0;
            }

            source
        };

        let read = read_intel_log_file(chatlog.character_log, chatlog.file.path(), log_source.cursor)?;
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

pub(crate) fn read_intel_log_file(character_log: CharacterLog, filepath: &Path, mut cursor: u64) -> SolarResult<LogRead> {
    let mut entries: Vec<LogEntry> = vec![];

    let mut file = File::open(&filepath)
        .map_err(|e| SolarError::read(e, &filepath))?;
    file.seek(SeekFrom::Start(cursor))
        .map_err(|e| SolarError::read(e, &filepath))?;
    let file = DecodeReaderBytesBuilder::new()
        .encoding(Some(UTF_16LE))
        .build(file);
    let mut reader = BufReader::new(file);

    loop {
        let mut buf = String::new();
        let size = reader.read_line(&mut buf)
            .map_err(|e| SolarError::read(e, &filepath))?;

        if size == 0 || buf.chars().last() != Some('\n') { break }
        let transcode_size = buf.encode_utf16().count() * 2;
        cursor += transcode_size as u64;

        let line = buf.trim().trim_start_matches('\u{feff}');
        if !line.starts_with('[') { continue }
        let pos = line.char_indices().nth(24).map(|(i,_)| i).unwrap_or(line.len());
        let Ok(stamp) = NaiveDateTime::parse_from_str(&line[..pos], "[ %Y.%m.%d %H:%M:%S ] ") else {
            continue
        };
        let datetime = stamp.and_utc();
        let line = &line[pos..];

        let Some(pos) = line.find(" > ") else { continue };
        let author = (&line[..pos]).to_string();
        let pos = line.char_indices().nth(pos + 3).map(|(i,_)| i).unwrap_or(line.len());
        let content = (&line[pos..]).to_string();

        let mut keywords = vec![];
        let mut systems = vec![];
        let words = content.split_whitespace();
        let mut num_words = 0;
        for word in words {
            num_words += 1;
            let word = word.trim_end_matches('*').to_uppercase();
            if let Some(keyword) = LogKeyword::matches(&word) {
                keywords.push(keyword);
            } else if let Some(system) = STAR_MAP.get_system_named(&word) {
                systems.push(system.id)
            }
        }

        let analysis = LogAnalysis::Intel(IntelLogAnalysis {
            ambiguous: num_words > (systems.len() + keywords.len()),
            systems,
            keywords,
        });

        let timestamp = datetime.into();
        let author = ChatAuthor::Character(author);

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
    pub fn display_ansi(&self, in_range: bool, danger: bool) -> LogEntryAnsi<'_> {
        LogEntryAnsi { entry: self, in_range, in_danger: danger }
    }
}

pub struct LogEntryAnsi<'a> {
    pub(crate) entry: &'a LogEntry,
    pub(crate) in_range: bool,
    pub(crate) in_danger: bool
}

pub struct LogEntryAnsiTheme<'a> {
    pub stamp_fg: Cow<'a, str>,
    pub author_fg: Cow<'a, str>,
    pub system_fg: Cow<'a, str>,
    pub content_fg: Cow<'a, str>,
}

impl<'a> LogEntryAnsiTheme<'a> {
    const IN_RANGE_AND_DANGER: Self = Self {
        stamp_fg: Cow::Borrowed(WHITE_ON_RED),
        author_fg: Cow::Borrowed(RED),
        system_fg: Cow::Borrowed(YELLOW),
        content_fg: Cow::Borrowed(RED),
    };
    const IN_RANGE_NO_DANGER: Self = Self {
        stamp_fg: Cow::Borrowed(WHITE_ON_ORANGE),
        author_fg: Cow::Borrowed(ORANGE),
        system_fg: Cow::Borrowed(YELLOW),
        content_fg: Cow::Borrowed(ORANGE),
    };
    const DEFAULT: Self = Self {
        stamp_fg: Cow::Borrowed(WHITE),
        author_fg: Cow::Borrowed(GRAY),
        system_fg: Cow::Borrowed(BRIGHT_BLUE),
        content_fg: Cow::Borrowed(WHITE),
    };

    pub fn from_entry<'b, 'c>(ansi: &'b LogEntryAnsi) -> &'c Self {
        match (ansi.in_range, ansi.in_danger) {
            (true, true) => &Self::IN_RANGE_AND_DANGER,
            (true, false) => &Self::IN_RANGE_NO_DANGER,
            (false, _) => &Self::DEFAULT,
        }
    }
}


impl<'a> std::fmt::Display for LogEntryAnsi<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let LogEntryAnsiTheme { stamp_fg, author_fg, system_fg, content_fg }
            = LogEntryAnsiTheme::from_entry(self);
        
        let stamp = self.entry.timestamp.to_datetime().format("%m-%d %H:%M");
        let author = format!("{:<37}", self.entry.author.as_str());
        let sys_names = self.entry.analysis.system_ids().iter()
            .map(|id| STAR_MAP.system(id).name)
            .collect::<Vec<_>>();

        let mut words = self.entry.content.split_whitespace()
            .map(|w| {
                let up = w.trim_end_matches('*').to_uppercase();
                if sys_names.contains(&up.as_str()) {
                    Phrase::System(Cow::Owned(up))
                } else {
                    Phrase::text(w)
                }
            })
            .collect::<Vec<_>>();

        // place system in the beginning, as god intended
        let sys_word = if self.entry.analysis.system_ids().len() == 1 {
            words.iter()
                .position(|w| w.is_system())
                .and_then(|pos| Some(format!("{system_fg}{} {CLR}", words.remove(pos))))
        } else {
            None
        }.unwrap_or_default();

        let content = words.into_iter()
            .map(|w| match (w, self.in_range) {
                (Phrase::System(s), _) => Cow::Owned(format!("{system_fg}{s}{CLR}")),
                (phrase, _) => phrase.take(),
            })
            .join(" ");

        let out = format!("{stamp_fg}{stamp}{CLR} {author_fg}{author}{CLR} {sys_word}{content_fg}{content}{CLR}");
        f.write_str(&out)
    }
}

enum Phrase<'a> {
    #[allow(dead_code)]
    Keyword(Cow<'a, str>),
    System(Cow<'a, str>),
    Text(Cow<'a, str>),
}

impl<'a> Phrase<'a> {
    #[allow(dead_code)]
    fn keyword(s: &'a str) -> Self { Self::Keyword(Cow::Borrowed(s)) }
    #[allow(dead_code)]
    fn system(s: &'a str) -> Self { Self::System(Cow::Borrowed(s)) }
    fn text(s: &'a str) -> Self { Self::Text(Cow::Borrowed(s)) }

    fn as_str(&'a self) -> &'a str {
        match self { Self::Keyword(s) | Self::System(s) | Self::Text(s) => s }
    }

    fn take(self) -> Cow<'a, str> {
        match self { Self::Keyword(s) | Self::System(s) | Self::Text(s) => s }
    }

    fn is_system(&self) -> bool { matches!(self, Self::System(_)) }
}

impl<'a> std::fmt::Display for Phrase<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_log_filename() {
        const FILENAME: &'static str = "our.intel_20260212_035141_12345678.txt";
        let expected = Some(ChatLogFile::new(PathBuf::from(FILENAME), |_| {
            let timestamp = Utc.with_ymd_and_hms(2026, 2, 12, 3, 51, 41)
                .single().unwrap()
                .into();
            ChatLogFileParts {
                channel: "our.intel",
                character_id: 12345678,
                timestamp, 
            }
        }));

        let actual = ChatLogFile::from_path_buf(PathBuf::from(FILENAME));
        assert_eq!(expected, actual);
    }
}

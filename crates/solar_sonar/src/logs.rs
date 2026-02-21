use crate::*;

#[derive(Debug)]
pub(crate) struct LogPath<'a> {
    path: PathBuf,
    channel: &'a str,
    datetime: DateTime<Utc>,
    character: &'a CharacterConfig,
}

#[derive(Debug)]
pub(crate) struct LogSource {
    file: PathBuf,
    cursor: u64,
}

#[derive(Debug)]
pub(crate) struct Log {
    pub channel: String,
    pub entries: Vec<LogEntry>,
    sources: HashMap<CharacterID, LogSource>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub(crate) struct LogEntry {
    pub datetime: DateTime<Utc>,
    pub author: String,
    pub content: String,
    pub analysis: LogAnalysis,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub(crate) struct LogAnalysis {
    pub systems: Vec<SolarID>,
    pub keywords: Vec<LogKeyword>,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub(crate) enum LogKeyword {
    Clear,
    NoVisual,
}

impl LogKeyword {
    pub fn matches(s: &str) -> Option<Self> {
        match s {
            "CLEAR" | "CLR" => Some(Self::Clear),
            "NV" => Some(Self::NoVisual),
            _ => None
        }
    }
}

pub(crate) struct ChannelLogs(HashMap<String, Log>);

impl ChannelLogs {
    pub fn new_watch(run: &Running) -> Self {
        let watch_chrs = run.args.watch_characters(&run.cfg);
        let logs = watch_chrs.iter()
            .map(|chr| &chr.intel_channels)
            .flatten()
            .map(|channel| Log { channel: channel.to_string(), entries: vec![], sources: HashMap::new() })
            .fold(HashMap::new(), |mut map, log| { map.insert(log.channel.to_string(), log); map });

        Self(logs)
    }

    pub fn get_mut(&mut self, channel: &str) -> Option<&mut Log> {
        self.0.get_mut(channel)
    }

    pub fn get(&self, channel: &str) -> Option<&Log> {
        self.0.get(channel)
    }

}

pub(crate) fn read_intel_logs(run: &Running, mut logs: ChannelLogs) -> SolarResult<ChannelLogs> {
    let chat_logs_dir = run.cfg.logs_dir.join("Chatlogs");
    let watch_chrs = run.args.watch_characters(&run.cfg);

    let log_paths = fs::read_dir(&chat_logs_dir)
        .map_err(|e| SolarError::list(e, &chat_logs_dir))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|ft| ft.is_file()))
        .filter_map(|entry| {
            let path = entry.path();
            if Some("txt") != path.extension().and_then(|p| p.to_str()) {
                return None
            }
            let Some(file) = path.file_stem().and_then(|p| p.to_str()) else {
                return None
            };

            for character in &watch_chrs {
                let Some(file) = file.strip_suffix(&character.id_str) else {
                    continue
                };
                let Some(file) = file.strip_suffix('_') else {
                    continue
                };

                for channel in &character.intel_channels {
                    let Some(file) = file.strip_prefix(channel) else {
                        continue
                    };
                    let Some(file) = file.strip_prefix('_') else {
                        continue
                    };
                    let Ok(stamp) = NaiveDateTime::parse_from_str(file, "%Y%m%d_%H%M%S") else {
                        continue
                    };
                    let datetime = stamp.and_utc();

                    return Some(LogPath { path, channel, datetime, character })
                }
            }

            None
        })
        .into_group_map_by(|log| (log.character.id, log.channel))
        .into_iter()
        .map(|(_,mut v)| {
            v.sort_by(|a,b| a.datetime.cmp(&b.datetime));
            v.pop().expect("value")
        })
        .collect::<Vec<_>>();

    for log_path in log_paths {
        let log = &mut logs.get_mut(log_path.channel).expect("log exists");
        let log_source = {
            let source = log.sources
                .entry(log_path.character.id)
                .or_insert(LogSource { file: log_path.path.clone(), cursor: 0 });

            if source.file != log_path.path {
                source.file = log_path.path.clone();
                source.cursor = 0;
            }

            source
        };

        let mut file = File::open(&log_path.path)
            .map_err(|e| SolarError::read(e, &log_path.path))?;
        file.seek(SeekFrom::Start(log_source.cursor))
            .map_err(|e| SolarError::read(e, &log_path.path))?;
        let file = DecodeReaderBytesBuilder::new()
            .encoding(Some(UTF_16LE))
            //.bom_sniffing(true)
            .build(file);
        let mut reader = BufReader::new(file);

        loop {
            let mut buf = String::new();
            let size = reader.read_line(&mut buf)
                .map_err(|e| SolarError::read(e, &log_path.path))?;

            if size == 0 || buf.chars().last() != Some('\n') { break }
            let transcode_size = buf.encode_utf16().count() * 2;
            log_source.cursor += transcode_size as u64;

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
            for word in content.split_whitespace() {
                let word = word.trim_end_matches('*').to_uppercase();
                if let Some(keyword) = LogKeyword::matches(&word) {
                    keywords.push(keyword);
                } else if let Some(system) = STAR_MAP.get_system_named(&word) {
                    systems.push(system.id)
                }
            }

            let analysis = LogAnalysis {
                systems,
                keywords,
            };


            let entry = LogEntry {
                datetime,
                author,
                content,
                analysis,
            };

            //dbg!(&entry);
            log.entries.push(entry);
        }
    }

    Ok(logs)
}

impl LogEntry {
    pub fn display_ansi(&self, alert: bool) -> LogEntryAnsi<'_> { LogEntryAnsi(self, alert) }
}

pub(crate) struct LogEntryAnsi<'a>(&'a LogEntry, bool);

impl<'a> std::fmt::Display for LogEntryAnsi<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stamp = self.0.datetime.format("%m-%d %H:%M");
        let author = format!("{:<37}", self.0.author);
        let sys_names = self.0.analysis.systems.iter()
            .map(|id| STAR_MAP.system(id).name)
            .collect::<Vec<_>>();

        let alert = self.1;
        let (stamp_color, author_color, sys_color, text_color) = if alert {
            (WHITE_ON_RED, RED, YELLOW, RED)
        } else {
            (WHITE, GRAY, BRIGHT_BLUE, WHITE)
        };

        let mut words = self.0.content.split_whitespace()
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
        let sys_word = if self.0.analysis.systems.len() == 1 {
            words.iter()
                .position(|w| w.is_system())
                .and_then(|pos| Some(format!("{sys_color}{} {CLR}", words.remove(pos))))
        } else {
            None
        }.unwrap_or_default();

        let content = words.into_iter()
            .map(|w| match (w, alert) {
                (Phrase::System(s), _) => Cow::Owned(format!("{sys_color}{s}{CLR}")),
                (phrase, _) => phrase.take(),
            })
            .join(" ");

        let out = format!("{stamp_color}{stamp}{CLR} {author_color}{author}{CLR} {sys_word}{text_color}{content}{CLR}");
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

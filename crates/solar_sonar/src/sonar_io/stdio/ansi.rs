use crate::*;

pub struct LogEntryAnsi<'this,'run:'this> {
    pub(crate) entry: &'this LogEntry,
    pub(crate) index: &'run Index,
    pub(crate) in_range: bool,
    pub(crate) in_danger: bool,
    pub(crate) dangerous: bool,
}

pub struct LogEntryAnsiTheme<'a> {
    pub stamp_fg: Cow<'a, str>,
    pub author_fg: Cow<'a, str>,
    pub focus_fg: Cow<'a, str>,
    pub content_fg: Cow<'a, str>,
}

impl<'a> LogEntryAnsiTheme<'a> {
    const IN_DANGER: Self = Self {
        stamp_fg: Cow::Borrowed(WHITE_ON_RED),
        author_fg: Cow::Borrowed(RED),
        focus_fg: Cow::Borrowed(YELLOW),
        content_fg: Cow::Borrowed(RED),
    };
    const IN_RANGE_NO_DANGER: Self = Self {
        stamp_fg: Cow::Borrowed(WHITE_ON_ORANGE),
        author_fg: Cow::Borrowed(ORANGE),
        focus_fg: Cow::Borrowed(YELLOW),
        content_fg: Cow::Borrowed(ORANGE),
    };
    const DEFAULT: Self = Self {
        stamp_fg: Cow::Borrowed(WHITE),
        author_fg: Cow::Borrowed(GRAY),
        focus_fg: Cow::Borrowed(BRIGHT_BLUE),
        content_fg: Cow::Borrowed(WHITE),
    };

    pub fn from_entry<'b, 'c>(ansi: &'b LogEntryAnsi) -> &'c Self {
        match (ansi.in_range, ansi.in_danger, ansi.dangerous) {
            (_, true, _) => &Self::IN_DANGER,
            (true, false, false) => &Self::IN_RANGE_NO_DANGER,
            (true, false, true) => &Self::IN_DANGER,
            _ => &Self::DEFAULT,
        }
    }
}

impl<'this,'run:'this> std::fmt::Display for LogEntryAnsi<'this,'run> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let LogEntryAnsiTheme { stamp_fg, author_fg, focus_fg, content_fg }
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

        let focus_word = if self.entry.analysis.has_single_system() {
            // place system in the beginning, as god intended
            words.iter()
                .position(|w| w.is_system())
                .and_then(|pos| Some(format!("{focus_fg}{}{CLR} ", words.remove(pos))))
        } else if self.entry.analysis.has_callout() {
            let chan = self.entry.character_log.display_str(self.index);
            Some(format!("{focus_fg}{chan}{CLR} "))
        } else {
            None
        }.unwrap_or_default();

        let content = words.into_iter()
            .map(|w| match (w, self.in_range) {
                (Phrase::System(s), _) => Cow::Owned(format!("{focus_fg}{s}{CLR}")),
                (phrase, _) => phrase.take(),
            })
            .join(" ");

        let out = format!("{stamp_fg}{stamp}{CLR} {author_fg}{author}{CLR} {focus_word}{content_fg}{content}{CLR}");
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

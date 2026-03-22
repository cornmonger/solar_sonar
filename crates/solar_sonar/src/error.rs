use crate::*;

#[derive(Debug, snafu::Snafu)]
pub enum SolarError {
    #[snafu(display("{msg}"))]
    Msg { msg: String },
    #[snafu(display("Failed to {op}: {p}",
        p = file.to_string_lossy(),
    ))]
    FileIO {
        source: io::Error,
        op: IoOp,
        file: PathBuf,
    },
    #[snafu(display("Failed to expand path variables: {p}", p=path.to_string_lossy()))]
    Path {
        source: shellexpand::path::LookupError<env::VarError>,
        path: PathBuf,
    },
    #[snafu(display("Failed to parse config: {p}", p=file.to_string_lossy()))]
    Cfg {
        source: toml::de::Error,
        file: PathBuf,
    },
    #[snafu(display("Failed to send event I/O"))]
    Send {
    },
    NotFound { noun: ErrNoun },
    Duplicate { noun: ErrNoun, item: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrNoun {
    Mode,
    ChatChannel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoOp {
    Read,
    Write,
    List,
    Path,
    MakeDir,
}

impl std::fmt::Display for IoOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path => f.write_str("parse path"),
            Self::Read => f.write_str("read file"),
            Self::Write => f.write_str("write file"),
            Self::List => f.write_str("list directory"),
            Self::MakeDir => f.write_str("make directory"),
        }
    }
}

pub type SolarResult<T> = Result<T, SolarError>;

impl SolarError {
    pub(crate) fn msg<S: Into<String>>(msg: S) -> Self {
        Self::Msg { msg: msg.into() }
    }

    pub(crate) fn err_msg<T, S: Into<String>>(msg: S) -> SolarResult<T> {
        Err(Self::Msg { msg: msg.into() })
    }

    pub(crate) fn cfg<P: AsRef<Path> + Into<PathBuf>>(source: toml::de::Error, file: P) -> Self {
        let file = short_path(file);
        Self::Cfg { source, file }
    }

    
    pub(crate) fn read<P: AsRef<Path> + Into<PathBuf>>(source: io::Error, file: P) -> Self {
        let file = short_path(file);
        Self::FileIO { source, op: IoOp::Read, file }
    }

    pub(crate) fn write<P: AsRef<Path>>(source: io::Error, file: P) -> Self {
        let file = short_path(file.as_ref());
        Self::FileIO { source, op: IoOp::Write, file }
    }

    pub(crate) fn list<P: AsRef<Path>>(source: io::Error, file: P) -> Self {
        let file = short_path(file.as_ref());
        Self::FileIO { source, op: IoOp::List, file }
    }

    pub(crate) fn mkdir<P: AsRef<Path>>(source: io::Error, file: P) -> Self {
        let file = short_path(file.as_ref());
        Self::FileIO { source, op: IoOp::MakeDir, file }
    }

    pub(crate) fn path<P: Into<PathBuf>>(source: shellexpand::path::LookupError<std::env::VarError>, path: P) -> Self {
        Self::Path { source, path: path.into() }
    }

    pub(crate) fn broadcast<T>(_source: tokio::sync::broadcast::error::SendError<T>) -> Self {
        Self::Send {}
    }
    
    pub(crate) fn not_found(noun: ErrNoun) -> Self {
        Self::NotFound { noun }
    }
    
    pub(crate) fn duplicate<S: Into<String>>(noun: ErrNoun, item: S) -> Self {
        Self::Duplicate { noun, item: item.into() }
    }
    
    pub(crate) fn err_duplicate<S: Into<String>, T>(noun: ErrNoun, item: S) -> SolarResult<T> {
        Err(Self::duplicate(noun, item))
    }
}

impl From<tokio::sync::broadcast::error::SendError<DataEvent>> for SolarError {
    fn from(source: tokio::sync::broadcast::error::SendError<DataEvent>) -> Self {
        Self::broadcast(source)
    }
}
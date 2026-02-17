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
    #[snafu(display("Failed to parse config: {p}", p=file.to_string_lossy()))]
    Cfg {
        source: toml::de::Error,
        file: PathBuf,
    },
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoOp {
    Read,
    Write,
    List,
    MakeDir,
}

impl std::fmt::Display for IoOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
}

use crate::*;

#[derive(Debug, PartialEq, Eq)]
pub enum ReplaySourceFile {
    Log(LogFile),
    Relog(RelogFormat, PathBuf),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReplaySource {
    LogDir(PathBuf),
    LogFile(LogFile),
    RelogDir(PathBuf),
    RelogFile(RelogFormat, PathBuf),
}

impl ReplaySource {
    pub fn try_from_path(path: PathBuf) -> SolarResult<Self> {
        if path.is_dir() {
            if path.join("Gamelogs").is_dir() || path.join("Chatlogs").is_dir() {
                Ok(Self::LogDir(path))
            } else {
                Ok(Self::RelogDir(path))
            }
        } else if path.ends_with(".txt") {
            let dir_kind = LogDirKind::from_file_path(&path)
                .ok_or_else(|| SolarError::msg(format!("Unable to determine log kind from parent directory: {}", path.to_string_lossy())))?;
            let log_file = LogFile::from_path_buf(path, dir_kind)
                .ok_or_else(|| SolarError::msg(format!("Unable to parse log filename")))?;
            Ok(Self::LogFile(log_file))
        } else if path.ends_with(".bin.log") {
            Ok(Self::RelogFile(RelogFormat::Binary, path))
        } else if path.ends_with(".ron.log") {
            Ok(Self::RelogFile(RelogFormat::Ron, path))
        } else {
            SolarError::err_msg(format!("Invalid replay source path: {}", path.to_string_lossy()))
        }
    }
    
    pub fn path(&self) -> &Path {
        match self {
            Self::LogFile(logfile) => logfile.path(),
            Self::LogDir(path) | Self::RelogDir(path) | Self::RelogFile(_, path) => path
        }
    }
}
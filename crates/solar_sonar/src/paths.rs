use crate::*;

pub(crate) const SOLAR_SUBDIR: &'static str = "solar_sonar";

pub(crate) fn data_dir() -> SolarResult<PathBuf> {
    let dir = SolarSonar::get().dirs.data_dir().join(SOLAR_SUBDIR);
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| SolarError::mkdir(e, "Unable to make data directory"))?;
    }

    Ok(dir)
}

/// SAFETY: panics
pub(crate) fn short_path<P: AsRef<Path> + Into<PathBuf>>(path: P) -> PathBuf {
    let path = path.as_ref();
    match path.strip_prefix(SolarSonar::get().dirs.home_dir()) {
        Ok(p) => Path::new("~/").join(p),
        Err(_) => path.into(),
    }
}

pub(crate) fn log_path<P: AsRef<Path> + Into<PathBuf>>(path: P) -> String {
    short_path(path).to_string_lossy().to_string()
}

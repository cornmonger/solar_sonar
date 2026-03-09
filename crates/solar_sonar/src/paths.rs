use crate::*;

pub(crate) const SOLAR_SUBDIR: &'static str = "solar_sonar";

/// SAFETY: panics
pub(crate) fn short_path<P: AsRef<Path> + Into<PathBuf>>(path: P) -> PathBuf {
    let path = path.as_ref();
    match path.strip_prefix(SolarSonar::get().tilde_dir()) {
        Ok(p) => Path::new("~/").join(p),
        Err(_) => path.into(),
    }
}

pub(crate) fn log_path<P: AsRef<Path> + Into<PathBuf>>(path: P) -> String {
    short_path(path).to_string_lossy().to_string()
}

pub(crate) fn expand_path(path: &Path) -> SolarResult<Cow<'_, Path>> {
    shellexpand::path::full(path)
        .map_err(|e| SolarError::path(e, path))
}

pub(crate) fn expand_pathbuf<P: AsRef<Path>>(path: P) -> SolarResult<PathBuf> {
    shellexpand::path::full(path.as_ref())
        .map(|p| p.to_path_buf())
        .map_err(|e| SolarError::path(e, path.as_ref()))
}

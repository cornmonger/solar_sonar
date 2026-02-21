use crate::*;

pub(crate) struct RemoteAsset {
    pub name: &'static str,
    pub dir: AssetDir,
    pub kind: AssetKind,
    pub filename: &'static str,
    pub default_url: &'static str,
}

impl RemoteAsset {
    pub(crate) fn filepath(&self) -> SolarResult<PathBuf> {
        Ok(self.dir.dir()?.join(self.filename))
    }

    pub(crate) fn download(&self) -> SolarResult<PathBuf> {
        let filepath = self.filepath()?;
        let mut resp = reqwest::blocking::get(self.default_url)
            .map_err(|_| SolarError::msg(format!("Failed to download: {}", self.default_url)))?
            .error_for_status()
            .map_err(|_| SolarError::msg(format!("Failed to download: {}", self.default_url)))?;
        let mut dest = File::create(&filepath)
            .map_err(|e| SolarError::write(e, &filepath))?;
        io::copy(&mut resp, &mut dest)
            .map_err(|e| SolarError::write(e, &filepath))?;

        match self.kind {
            AssetKind::File => Ok(filepath),
            AssetKind::ZipDir => self.unzip_dir(),
        }
    }

    pub(crate) fn dirpath(&self) -> SolarResult<PathBuf> {
        assert!(self.kind == AssetKind::ZipDir);

        let filepath = self.filepath()?;
        let dirname = filepath.file_prefix()
            .ok_or_else(|| SolarError::msg(format!("Unable to determine dirname for asset: {}", log_path(&filepath))))?;

        let dirpath = self.dir.dir()?.join(dirname);
        Ok(dirpath)
    }

    fn unzip_dir(&self) -> SolarResult<PathBuf> {
        let filepath = self.filepath()?;
        let dirpath = self.dirpath()?;
        if !dirpath.exists() {
            fs::create_dir_all(&dirpath)
                .map_err(|e| SolarError::mkdir(e, &dirpath))?;
        }

        let file = File::open(&filepath)
            .map_err(|e| SolarError::write(e, &filepath))?;
        let decoder = BzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(self.dir.dir()?)
            .map_err(|_| SolarError::msg(format!("Failed to unzip: {}", log_path(filepath))))?;

        Ok(dirpath)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AssetDir {
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AssetKind {
    File,
    ZipDir,
}

impl AssetDir {
    pub(crate) fn dir(&self) -> SolarResult<PathBuf> {
        match self {
            Self::Data => data_dir(),
        }
    }
}

#[allow(dead_code)]
pub(crate) enum RemoteAssets {
    EspeakData,
    VoiceModelConfig,
    VoiceModel,
}

impl RemoteAssets {
    pub(crate) const REMOTE_ASSETS: &'static [RemoteAsset] = &[
        // Source: https://huggingface.co/rhasspy/piper-voices
        // Path: https://huggingface.co/rhasspy/piper-voices/tree/main/en/en_US/libritts_r/medium
        // File: https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx.json
        // Info: https://github.com/rhasspy/piper/blob/master/VOICES.md
        // License: MIT
        RemoteAsset {
            name: "Voice Model Configuration",
            dir: AssetDir::Data,
            kind: AssetKind::File,
            filename: "en_US-libritts_r-medium.onnx.json",
            default_url: "https://github.com/cornmonger/solar_sonar_assets/raw/refs/heads/dev/thirdparty/rhasspy/piper-voices/en_US-libritts_r-medium.onnx.json",
        },
        // Source: https://huggingface.co/rhasspy/piper-voices
        // Path: https://huggingface.co/rhasspy/piper-voices/tree/main/en/en_US/libritts_r/medium
        // File: https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx
        // Info: https://github.com/rhasspy/piper/blob/master/VOICES.md
        // License: MIT
        RemoteAsset {
            name: "Voice Model",
            dir: AssetDir::Data,
            kind: AssetKind::File,
            filename: "en_US-libritts_r-medium.onnx",
            default_url: "https://media.githubusercontent.com/media/cornmonger/solar_sonar_assets/refs/heads/dev/thirdparty/rhasspy/piper-voices/en_US-libritts_r-medium.onnx",
        },
        // Source: https://github.com/espeak-ng/espeak-ng
        //    via project: piper-rs
        // License: GPL3
        RemoteAsset {
            name: "ESpeak Data",
            dir: AssetDir::Data,
            kind: AssetKind::ZipDir,
            filename: "espeak_data.tar.bz2",
            default_url: "https://media.githubusercontent.com/media/cornmonger/solar_sonar_assets/refs/heads/dev/thirdparty/espeak/espeak_data.tar.bz2",
        }
    ];

    pub const fn get(&self) -> &'static RemoteAsset {
        match self {
            Self::VoiceModelConfig => &Self::REMOTE_ASSETS[0],
            Self::VoiceModel => &Self::REMOTE_ASSETS[1],
            Self::EspeakData => &Self::REMOTE_ASSETS[2],
        }
    }

    pub const fn all() -> &'static [RemoteAsset] { Self::REMOTE_ASSETS }
}


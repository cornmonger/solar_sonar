use crate::*;

pub(crate) struct RemoteAsset {
    pub name: &'static str,
    pub dir: AssetDir,
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

        Ok(filepath)
    }
}

pub(crate) enum AssetDir {
    Data,
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
            filename: "en_US-libritts_r-medium.onnx.json",
            default_url: "https://github.com/cornmonger/solar_sonar_asssets/raw/refs/heads/dev/thirdparty/rhasspy/piper-voices/en_US-libritts_r-medium.onnx.json",
        },
        // Source: https://huggingface.co/rhasspy/piper-voices
        // Path: https://huggingface.co/rhasspy/piper-voices/tree/main/en/en_US/libritts_r/medium
        // File: https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx
        // Info: https://github.com/rhasspy/piper/blob/master/VOICES.md
        // License: MIT
        RemoteAsset {
            name: "Voice Model",
            dir: AssetDir::Data,
            filename: "en_US-libritts_r-medium.onnx",
            default_url: "https://media.githubusercontent.com/media/cornmonger/solar_sonar_asssets/refs/heads/dev/thirdparty/rhasspy/piper-voices/en_US-libritts_r-medium.onnx",
        },
    ];

    pub const fn get(&self) -> &'static RemoteAsset {
        match self {
            Self::VoiceModelConfig => &Self::REMOTE_ASSETS[0],
            Self::VoiceModel => &Self::REMOTE_ASSETS[1],
        }
    }

    pub const fn all() -> &'static [RemoteAsset] { Self::REMOTE_ASSETS }
}


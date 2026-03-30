use crate::*;

#[derive(Debug, PartialEq, Eq)]
pub struct ModeConfig {
    pub id: ModeId,
    pub combat_analysis: CombatAnalysisModeConfig,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ModeDirCfg {
    pub combat_analysis: CombatAnalysisModeCfg,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct NamedModeDirCfg {
    pub name: String,
    pub combat_analysis: CombatAnalysisModeCfg,
}

impl ModeConfig {
    pub(crate) fn try_from_cfg(cfg: NamedModeDirCfg, mode_index: &ModeIndex) -> SolarResult<Self> {
        let id = mode_index.find(&cfg.name)?.id();
        let combat_analysis = CombatAnalysisModeConfig::try_from_cfg(cfg.combat_analysis)?;
        
        Ok(Self {
            id,
            combat_analysis,
        })
    }
}

impl NamedModeDirCfg {
    pub fn read_dir(config_dir: &Path, mode_name: &str) -> SolarResult<Self> {
        let combat_analysis = CombatAnalysisModeCfg::read(config_dir, mode_name)?;
        let name = mode_name.to_string();
        
        Ok(Self {
            name,
            combat_analysis,
        })
    }
}
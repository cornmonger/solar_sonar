use crate::*;

#[derive(Debug, PartialEq, Eq)]
pub struct ModeConfig {
    pub id: ModeId,
    pub combat_analysis: CombatAnalysisModeConfig,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ModeCfg {
    pub combat_analysis: CombatAnalysisModeCfg,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct NamedModeCfg {
    pub name: String,
    pub cfg: ModeCfg,
}

impl ModeConfig {
    pub(crate) fn try_from_cfg(cfg: NamedModeCfg, mode_index: &ModeIndex) -> SolarResult<Self> {
        let id = mode_index.find(&cfg.name)?.id();
        let combat_analysis = CombatAnalysisModeConfig::try_from_cfg(cfg.cfg.combat_analysis)?;
        
        Ok(Self {
            id,
            combat_analysis,
        })
    }
}

impl NamedModeCfg {
    pub fn read_dir(config_dir: &Path, mode_name: &str) -> SolarResult<Self> {
        let combat_analysis = CombatAnalysisModeCfg::read(config_dir, mode_name)?;
        let name = mode_name.to_string();
        let cfg = ModeCfg {
            combat_analysis,
        };
        
        Ok(Self {
            name,
            cfg,
        })
    }
}

#[derive(Debug)]
pub struct NamedModeCfgConst {
    pub name: &'static str,
    pub combat_analysis: CombatAnalysisModeCfgConst,
}

impl From<&NamedModeCfgConst> for NamedModeCfg {
    fn from(v: &NamedModeCfgConst) -> Self {
        Self {
            name: v.name.to_string(),
            cfg: ModeCfg {
                combat_analysis: (&v.combat_analysis).into()
            },
        }
    }
}

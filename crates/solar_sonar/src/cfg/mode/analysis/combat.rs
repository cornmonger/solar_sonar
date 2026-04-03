use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatAnalysisModeConfig {
    /// in seconds
    pub cease_time: u16,
    pub engagement_kind: EngagementKind,
    pub targeted_kind: TargetedKind,
    pub yellowbox_threshold: u8,
}

impl CombatAnalysisModeConfig {
    pub(crate) const DEFAULT: Self = Self {
        cease_time: 90,
        engagement_kind: EngagementKind::PVP,
        targeted_kind: TargetedKind::Watched,
        yellowbox_threshold: 3,
    };
}

impl Default for CombatAnalysisModeConfig {
    #[inline]
    fn default() -> Self { Self::DEFAULT }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum TargetedKind {
    Watched,
    Alliance,
    Any,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum EngagementKind {
    PVP,
    Any,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CombatAnalysisModeCfg {
    pub cease_time: Option<u16>,
    pub engagement: Option<EngagementKind>,
    pub targeted: Option<TargetedKind>,
    pub yellowboxers: Option<u8>,
}

impl ModalCfgToml for CombatAnalysisModeCfg {
    const SUBDIR: Option<&'static str> = MODE_SUBDIR_ANALYSIS; 
    const TOML_FILENAME: &'static str = "combat.toml";
    const DEFAULT_TOML: &'static str = include_str!("../../../../assets/config/default/modal/analysis/combat.toml");
}

impl CombatAnalysisModeConfig {
    pub fn try_from_cfg(cfg: CombatAnalysisModeCfg) -> SolarResult<Self> {
        let cease_time = cfg.cease_time.unwrap_or(Self::DEFAULT.cease_time);
        let engagement_kind = cfg.engagement.unwrap_or(Self::DEFAULT.engagement_kind);
        let targeted_kind = cfg.targeted.unwrap_or(Self::DEFAULT.targeted_kind);
        let yellowbox_threshold = cfg.yellowboxers.unwrap_or(Self::DEFAULT.yellowbox_threshold);
        
        Ok(Self {
            cease_time,
            engagement_kind,
            targeted_kind,
            yellowbox_threshold,
        })
    }
}

#[derive(Debug)]
pub struct CombatAnalysisModeCfgConst {
    pub cease_time: Option<u16>,
    pub engagement: Option<EngagementKind>,
    pub targeted: Option<TargetedKind>,
    pub yellowboxers: Option<u8>,
}

impl From<&CombatAnalysisModeCfgConst> for CombatAnalysisModeCfg {
    fn from(v: &CombatAnalysisModeCfgConst) -> Self {
        Self {
            cease_time: v.cease_time,
            engagement: v.engagement,
            targeted: v.targeted,
            yellowboxers: v.yellowboxers,
        }
    }
}


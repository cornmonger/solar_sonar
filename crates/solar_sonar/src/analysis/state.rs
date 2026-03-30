use crate::*;

pub(crate) struct AnalyzerState {
    pub(crate) combat: CombatAnalyzerState,
}

impl AnalyzerState {
    pub(crate) fn new() -> Self { Self { combat: CombatAnalyzerState::new() } }
}
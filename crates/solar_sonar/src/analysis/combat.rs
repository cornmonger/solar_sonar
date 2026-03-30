use crate::*;

#[derive(
    Debug, Clone, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub struct CombatAnalysis {
    pub combat: Option<Combat>,
}

#[derive(
    Debug, Clone, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub struct Combat {
    /// pvp or pve
    pub kind: CombatKind,
    /// player or npc
    pub attacker: Combatant,
    /// player or npc
    pub target: Combatant,
    /// attacker and target are in the same alliance / corp
    pub friendly_fire: bool,
}

#[derive(
    Debug, Clone, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub enum CombatKind {
    /// Player vs Player
    PVP,
    /// Player vs Environment
    PVE,
}

#[derive(
    Debug, Clone, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub enum Combatant {
    KnownCharacter(CharacterId),
    Character {
        name: String,
        corporation: String,
        alliance: String,
    },
    /// Non-Player Character
    NPC {
        name: String,
    },
}

impl Analysis for CombatAnalysis {
    fn dangerous(&self) -> bool {
        self.combat.as_ref().is_some_and(|combat| {
            combat.kind == CombatKind::PVP
            && combat.friendly_fire == false 
        })
    }
}

pub struct CombatAnalyzer<'a> {
    cfg: &'a CombatAnalysisModeConfig,
    state: &'a mut CombatAnalyzerState,
}

pub(crate) struct CombatAnalyzerState {
    last_combat_time: Timestamp,
    last_yellowbox_time: Timestamp,
}

impl CombatAnalyzerState {
    pub(crate) fn new() -> Self {
        Self {
            last_combat_time: Timestamp::zero(),
            last_yellowbox_time: Timestamp::zero(),
        }
    }
}

impl<'a> CombatAnalyzer<'a> {
    pub(crate) fn new(cfg: &'a ModeConfig, state: &'a mut AnalyzerState) -> Self {
        let cfg = &cfg.combat_analysis;
        let state = &mut state.combat;
        
        Self {
            cfg,
            state,
        }
    }
    
    pub(crate) fn analyze(mut self, content: &str, time: &Timestamp) -> CombatAnalysis {
        let combat = None;
        CombatAnalysis {
            combat,
        }
    }
}
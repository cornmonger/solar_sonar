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
    PvP,
    /// Player vs Environment
    PvE,
}

#[derive(
    Debug, Clone, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub enum Combatant {
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
            combat.kind == CombatKind::PvP
            && combat.friendly_fire == false 
        })
    }
}

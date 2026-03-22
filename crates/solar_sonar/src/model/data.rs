use crate::*;

pub type CharacterId = u32;

#[derive(Debug, Clone)]
pub struct StarMap {
    pub systems: &'static [SolarSystem],
}

impl StarMap {
    pub const fn get() -> &'static Self {
        &STAR_MAP
    }
    
    pub fn system(&'static self, id: &SolarId) -> &'static SolarSystem {
        self.systems.iter().find(|sys| &sys.id == id).expect("system exists")
    }
    
    pub fn systems<'i, I: IntoIterator<Item = SolarId>>(&'static self, system_ids: I) -> Vec<&'static SolarSystem> {
        system_ids.into_iter()
            .map(|sys_id| self.system(&sys_id))
            .collect::<Vec<_>>()
    }

    pub fn get_system(&'static self, id: SolarId) -> Option<&'static SolarSystem> {
        self.systems.iter().find(|sys| sys.id == id)
    }
    
    pub fn system_named(&'static self, name: &str) -> &'static SolarSystem {
        self.systems.iter().find(|sys| sys.name == name).expect("system exists")
    }

    pub fn get_system_named(&'static self, name: &str) -> Option<&'static SolarSystem> {
        self.systems.iter().find(|sys| sys.name == name)
    }
}

pub type SolarId = u32;

#[derive(Debug, Clone)]
pub struct SolarSystem {
    pub id: SolarId,
    pub name: &'static str,
    pub gates: &'static [SolarId],
}

pub struct StarNavigator {
    pub star_map: &'static StarMap,
}

impl Default for StarNavigator {
    fn default() -> Self {
        Self { star_map: &STAR_MAP }
    }
}

impl StarNavigator {
    pub fn systems_in_range(&self, jumps: u8, from_system: &'static SolarSystem) -> Vec<&'static SolarSystem> {
        if jumps <= 0 {
            return vec![];
        }

        let gate_system_ids = from_system.gates.to_vec();

        if jumps == 1 {
            return gate_system_ids.into_iter()
                .map(|sys_id| self.star_map.system(&sys_id))
                .collect();
        }

        let mut range = gate_system_ids.clone();

        for gate_system_id in gate_system_ids {
            let gate_system = self.star_map.system(&gate_system_id);
            range = self.next_jump_range(jumps - 1, gate_system, range);
        }

        range.into_iter()
            .map(|sys_id| self.star_map.system(&sys_id))
            .collect()
    }

    fn next_jump_range(&self, jumps: u8, from_system: &'static SolarSystem, mut range: Vec<SolarId>) -> Vec<SolarId> {
        let gate_system_ids = from_system.gates.iter()
            .filter(|sys_id| !range.contains(sys_id))
            .collect::<Vec<_>>();

        range.extend(gate_system_ids.clone());

        if jumps <= 1 {
            return range;
        }

        for gate_system_id in gate_system_ids {
            let gate_system = self.star_map.system(gate_system_id);
            range = self.next_jump_range(jumps - 1, gate_system, range);
        }

        range
    }
}

pub type ChannelId = u64;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash,
    serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
#[repr(u8)]
pub enum LogKind {
    Game,
    Chat(ChatLogKind),
}

impl LogKind {
    pub fn try_from_enum(s: String) -> SolarResult<Self> {
        if let Ok(chat_kind) = ChatLogKind::try_from_enum(&s) {
            Ok(Self::Chat(chat_kind))
        } else if s.to_lowercase() == "game" {
            Ok(Self::Game)
        } else {
            Err(SolarError::msg("Invalid log kind"))
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash,
    serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
#[repr(u8)]
pub enum ChatLogKind {
    Alliance,
    Corporation,
    Fleet,
    Group,
    Local,
    Private,
}

impl ChatLogKind {
    pub fn try_from_enum(s: &str) -> SolarResult<Self> {
        match s.to_lowercase().as_str() {
            "alliance" => Ok(Self::Alliance),
            "corp" => Ok(Self::Corporation),
            "local" => Ok(Self::Local),
            "fleet" => Ok(Self::Fleet),
            _ => Err(SolarError::msg("Invalid chat log kind")),
        }
    }
    
    pub fn from_log_name(name: &str) -> Self {
        match name {
            "Alliance" => Self::Alliance,
            "Corp" => Self::Corporation,
            "Local" => Self::Local,
            "Fleet" => Self::Fleet,
            "Private" => Self::Private,
            _ => Self::Group,
        }
    }
}

use crate::*;

pub type CharacterID = u32;

#[derive(Debug, Clone)]
pub struct StarMap {
    pub systems: &'static [SolarSystem],
}

impl StarMap {
    pub const fn get() -> &'static Self {
        &STAR_MAP
    }
    
    pub fn system(&'static self, id: &SolarID) -> &'static SolarSystem {
        self.systems.iter().find(|sys| &sys.id == id).expect("system exists")
    }
    
    pub fn systems<'i, I: IntoIterator<Item = SolarID>>(&'static self, system_ids: I) -> Vec<&'static SolarSystem> {
        system_ids.into_iter()
            .map(|sys_id| self.system(&sys_id))
            .collect::<Vec<_>>()
    }

    pub fn get_system(&'static self, id: SolarID) -> Option<&'static SolarSystem> {
        self.systems.iter().find(|sys| sys.id == id)
    }
    
    pub fn system_named(&'static self, name: &str) -> &'static SolarSystem {
        self.systems.iter().find(|sys| sys.name == name).expect("system exists")
    }

    pub fn get_system_named(&'static self, name: &str) -> Option<&'static SolarSystem> {
        self.systems.iter().find(|sys| sys.name == name)
    }
}

pub type SolarID = u32;

#[derive(Debug, Clone)]
pub struct SolarSystem {
    pub id: SolarID,
    pub name: &'static str,
    pub gates: &'static [SolarID],
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

    fn next_jump_range(&self, jumps: u8, from_system: &'static SolarSystem, mut range: Vec<SolarID>) -> Vec<SolarID> {
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

pub type ChatChannelId = u64;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash,
    serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
#[repr(u8)]
pub enum LogKind {
    System,
    Chat(ChatLogKind),
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
    Group,
    Local,
    Fleet,
    Private,
}

impl ChatLogKind {
    pub fn from_name(name: &str) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatChannelRef<'a> {
    pub name: &'a str,
    pub id: ChatChannelId,
    pub kind: ChatLogKind,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChatChannel {
    pub name: String,
    pub id: ChatChannelId,
    pub kind: ChatLogKind,
}

impl std::hash::Hash for ChatChannel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<'a> From<&'a ChatChannel> for ChatChannelRef<'a> {
    fn from(v: &'a ChatChannel) -> Self {
        Self {
            name: v.name.as_str(),
            id: v.id,
            kind: v.kind,
        }
    }
}

impl<'a> Display for ChatChannelRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChatChannels(HashSet<ChatChannel>);
impl ChatChannels {
    pub fn new(channels: Vec<String>) -> Self {
        let channels = channels.into_iter()
            .map(|s| ChatChannel {
                kind: ChatLogKind::from_name(&s),
                id: xxh3_64(s.as_bytes()),
                name: s,
            })
            .collect::<HashSet<_>>();

        Self(channels)
    }

    pub fn extend(&mut self, channels: ChatChannels) {
        self.0.extend(channels.0);
    }

    pub fn find_id(&self, id: ChatChannelId) -> Option<ChatChannelRef<'_>> {
        self.0.iter().find(|c| c.id == id).map(ChatChannelRef::from)
    }

    pub fn find_name(&self, name: &str) -> Option<ChatChannelRef<'_>> {
        self.0.iter().find(|c| c.name == name).map(ChatChannelRef::from)
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<'_, ChatChannel> {
        self.0.iter()
    }
}

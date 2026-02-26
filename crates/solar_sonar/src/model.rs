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

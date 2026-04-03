use crate::*;

pub struct StarNavigator {
    pub star_map: &'static StarMap,
}

impl Default for StarNavigator {
    fn default() -> Self {
        Self { star_map: &STAR_MAP }
    }
}

impl StarNavigator {
    pub fn systems_in_range(&self, jumps: u8, from_system: &'static StarSystem) -> Vec<&'static StarSystem> {
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

    fn next_jump_range(&self, jumps: u8, from_system: &'static StarSystem, mut range: Vec<StarId>) -> Vec<StarId> {
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

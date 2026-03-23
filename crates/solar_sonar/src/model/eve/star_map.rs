use crate::*;

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

#[derive(Debug, Clone)]
pub struct SolarSystem {
    pub id: SolarId,
    pub name: &'static str,
    pub gates: &'static [SolarId],
}

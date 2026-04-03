use crate::*;

#[derive(Debug, Clone)]
pub struct StarMap {
    pub systems: &'static [StarSystem],
}

impl StarMap {
    pub const fn get() -> &'static Self {
        &STAR_MAP
    }
    
    pub fn system(&'static self, id: &StarId) -> &'static StarSystem {
        self.systems.iter().find(|sys| &sys.id == id).expect("system exists")
    }
    
    pub fn systems<'i, I: IntoIterator<Item = StarId>>(&'static self, system_ids: I) -> Vec<&'static StarSystem> {
        system_ids.into_iter()
            .map(|sys_id| self.system(&sys_id))
            .collect::<Vec<_>>()
    }

    pub fn get_system(&'static self, id: StarId) -> Option<&'static StarSystem> {
        self.systems.iter().find(|sys| sys.id == id)
    }
    
    pub fn system_named(&'static self, name: &str) -> &'static StarSystem {
        self.systems.iter().find(|sys| sys.name == name).expect("system exists")
    }

    pub fn get_system_named(&'static self, name: &str) -> Option<&'static StarSystem> {
        self.systems.iter().find(|sys| sys.name == name)
    }
}

#[derive(Debug, Clone)]
pub struct StarSystem {
    pub id: StarId,
    pub name: &'static str,
    pub gates: &'static [StarId],
}

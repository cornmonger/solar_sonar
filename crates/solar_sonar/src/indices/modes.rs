use crate::*;

#[derive(Debug, PartialEq, Eq)]
pub struct ModeIdx {
    id: IndexId, 
    name: String,
}

impl ModeIdx {
    pub fn name(&self) -> &str { &self.name }
}

impl Idx for ModeIdx {
    fn id(&self) -> IndexId { self.id }
    fn key(&self) -> &str { self.name() }
}

impl Hash for ModeIdx {
    fn hash<H: Hasher>(&self, state: &mut H) { self.id().hash(state); }
}

impl Display for ModeIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.key())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModeIndex(Vec<ModeIdx>);

impl ModeIndex {
    pub(crate) fn try_new(modes: Vec<String>) -> SolarResult<Self> {
        let modes = Index::no_duplicate(modes, ErrNoun::Mode)?.into_iter()
            .map(|name| ModeIdx {
                id: Index::hash_id(&name),
                name
            })
            .collect::<Vec<_>>();
        
        Ok(Self(modes))
    }
}

impl IndexedInner<ModeIdx> for ModeIndex {
    const NOUN: ErrNoun = ErrNoun::Mode;
    fn inner(&self) -> &Vec<ModeIdx> { &self.0 }
    fn inner_mut(&mut self) -> &mut Vec<ModeIdx> { &mut self.0 }
    fn take_inner(&mut self) -> Vec<ModeIdx> { mem::take(&mut self.0) }
}

impl Indexed<ModeIdx> for ModeIndex {}
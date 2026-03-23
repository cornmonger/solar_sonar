use crate::*;

#[derive(Debug, PartialEq, Eq)]
pub struct CharacterIdx {
    id: IndexId, 
    /// key, from config
    alias: String,
    /// from eve
    character_id: CharacterId,
    /// from eve
    name: String,
}

impl CharacterIdx {
    pub fn alias(&self) -> &str { &self.alias }
    pub fn character_id(&self) -> CharacterId { self.character_id }
    pub fn character_name(&self) -> &str { &self.name }
}

impl Idx for CharacterIdx {
    fn id(&self) -> IndexId { self.id }
    fn key(&self) -> &str { self.alias() }
}

impl Hash for CharacterIdx {
    fn hash<H: Hasher>(&self, state: &mut H) { self.id().hash(state); }
}

impl Display for CharacterIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.key())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CharacterIndex(Vec<CharacterIdx>);

impl CharacterIndex {
    pub(crate) fn try_from_cfg(characters: &Vec<CharacterConfig>) -> SolarResult<Self> {
        let inner = Self::new_inner_from(characters, |id, alias, character| CharacterIdx { 
            id,
            alias,
            character_id: character.id,
            name: character.name.clone(),
        })?;
        
        Ok(Self(inner))
    }
}

impl IndexedInner<CharacterIdx> for CharacterIndex {
    const NOUN: ErrNoun = ErrNoun::Mode;
    fn inner(&self) -> &Vec<CharacterIdx> { &self.0 }
    fn inner_mut(&mut self) -> &mut Vec<CharacterIdx> { &mut self.0 }
}

impl Indexed<CharacterIdx> for CharacterIndex {}
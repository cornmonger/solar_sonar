use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChannelNameIdx {
    pub id: IndexId,
    pub name: String,
}

impl ChannelNameIdx {
    pub fn name(&self) -> &str { &self.name }
}

impl Idx for ChannelNameIdx {
    fn id(&self) -> IndexId { self.id }
    fn key(&self) -> &str { self.name() }
}

impl Hash for ChannelNameIdx {
    fn hash<H: Hasher>(&self, state: &mut H) { self.id().hash(state); }
}

impl Display for ChannelNameIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.key())
    }
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChannelNameIndex(Vec<ChannelNameIdx>);
impl ChannelNameIndex {
    pub fn try_new(channel_names: Vec<String>) -> SolarResult<Self> {
        let inner = Self::new_inner(channel_names, |id, name| ChannelNameIdx {
            id,
            name,
        })?;
        
        Ok(Self(inner))
    }
}

impl IndexedInner<ChannelNameIdx> for ChannelNameIndex {
    const NOUN: ErrNoun = ErrNoun::ChannelName;
    fn inner(&self) -> &Vec<ChannelNameIdx> { &self.0 }
    fn inner_mut(&mut self) -> &mut Vec<ChannelNameIdx> { &mut self.0 }
}

impl Indexed<ChannelNameIdx> for ChannelNameIndex {}
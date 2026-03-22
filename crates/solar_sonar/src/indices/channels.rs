use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChatChannelIdx {
    pub id: IndexId,
    pub name: String,
    pub kind: ChatLogKind,
}

impl ChatChannelIdx {
    pub fn name(&self) -> &str { &self.name }
}

impl Idx for ChatChannelIdx {
    fn id(&self) -> IndexId { self.id }
    fn key(&self) -> &str { self.name() }
}

impl Hash for ChatChannelIdx {
    fn hash<H: Hasher>(&self, state: &mut H) { self.id().hash(state); }
}

impl Display for ChatChannelIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.key())
    }
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChatChannelIndex(Vec<ChatChannelIdx>);
impl ChatChannelIndex {
    pub fn try_new(channels: Vec<String>) -> SolarResult<Self> {
        let inner = Self::new_inner(channels, |id, name| ChatChannelIdx {
            id,
            kind: ChatLogKind::from_log_name(&name),
            name,
        })?;
        
        Ok(Self(inner))
    }
}

impl IndexedInner<ChatChannelIdx> for ChatChannelIndex {
    const NOUN: ErrNoun = ErrNoun::ChatChannel;
    fn inner(&self) -> &Vec<ChatChannelIdx> { &self.0 }
    fn inner_mut(&mut self) -> &mut Vec<ChatChannelIdx> { &mut self.0 }
}

impl Indexed<ChatChannelIdx> for ChatChannelIndex {}
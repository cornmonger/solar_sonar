use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChatChannel {
    pub id: IndexId,
    pub name: String,
    pub kind: ChatLogKind,
}

impl ChatChannel {
    pub fn name(&self) -> &str { &self.name }
}

impl Idx for ChatChannel {
    fn id(&self) -> IndexId { self.id }
    fn key(&self) -> &str { self.name() }
}

impl Hash for ChatChannel {
    fn hash<H: Hasher>(&self, state: &mut H) { self.id().hash(state); }
}

impl Display for ChatChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChatChannels(Vec<ChatChannel>);
impl ChatChannels {
    pub fn try_new(channels: Vec<String>) -> SolarResult<Self> {
        let channels = Index::no_duplicate(channels, ErrNoun::ChatChannel)?
            .into_iter()
            .map(|s| ChatChannel {
                kind: ChatLogKind::from_log_name(&s),
                id: Index::hash_id(&s),
                name: s,
            })
            .collect::<Vec<_>>();

        Ok(Self(channels))
    }
}

impl IndexedInner<ChatChannel> for ChatChannels {
    const NOUN: ErrNoun = ErrNoun::ChatChannel;
    fn inner(&self) -> &Vec<ChatChannel> { &self.0 }
    fn inner_mut(&mut self) -> &mut Vec<ChatChannel> { &mut self.0 }
    fn take_inner(&mut self) -> Vec<ChatChannel> { mem::take(&mut self.0) }
}

impl Indexed<ChatChannel> for ChatChannels {}
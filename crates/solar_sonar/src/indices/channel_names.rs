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
    
    pub fn index_named(&mut self, named: NamedLog) -> IndexedLog {
        match named {
            NamedLog::Alliance => IndexedLog::Alliance,
            NamedLog::Corporation => IndexedLog::Corporation,
            NamedLog::Game => IndexedLog::Game,
            NamedLog::Local => IndexedLog::Local,
            NamedLog::Fleet => IndexedLog::Fleet,
            NamedLog::Group(name) => {
                let id = if let Ok(idx) = self.find(&name) {
                    idx.id()
                } else {
                    self.0.push(ChannelNameIdx {
                        id: Index::hash_id(&name),
                        name,
                    });
                    
                    self.0.get(self.0.len() - 1)
                        .expect("pushed")
                        .id()
                };
                
                IndexedLog::Group(id)
            },
        }
    }
}

impl IndexedInner<ChannelNameIdx> for ChannelNameIndex {
    const NOUN: ErrNoun = ErrNoun::ChannelName;
    fn inner(&self) -> &Vec<ChannelNameIdx> { &self.0 }
    fn inner_mut(&mut self) -> &mut Vec<ChannelNameIdx> { &mut self.0 }
}

impl Indexed<ChannelNameIdx> for ChannelNameIndex {}
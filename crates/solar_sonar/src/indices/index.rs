use crate::*;

pub type IndexId = u64;

pub trait Idx: Sized + Hash + Display {
    fn id(&self) -> IndexId;
    fn key(&self) -> &str;
}


#[derive(Debug, PartialEq, Eq)]
pub struct Index {
    modes: ModeIndex,
    chat_channels: ChatChannelIndex,
}

impl Index {
    pub fn new(modes: ModeIndex, chat_channels: ChatChannelIndex) -> Self {
        Self { modes, chat_channels}
    }
    
    pub fn hash_id(s: &str) -> IndexId {
        xxh3_64(s.as_bytes())
    }
    
    pub fn modes(&self) -> &ModeIndex { &self.modes }
    pub fn chat_channels(&self) -> &ChatChannelIndex { &self.chat_channels }
    
    pub fn chat_channels_mut(&mut self) -> &mut ChatChannelIndex {
        &mut self.chat_channels
    }
    
    pub(crate) fn no_duplicate(v: Vec<String>, noun: ErrNoun) -> SolarResult<Vec<String>> {
        for s in &v {
            if v.iter().filter(|si| si.as_str() == s).count() > 1 {
                return SolarError::err_duplicate(noun, s);
            }
        }
        
        Ok(v)
    }
    
}

pub(crate) trait IndexedInner<IDX: Idx>: Sized {
    const NOUN: ErrNoun;
    
    fn new_inner(inner: Vec<String>, build: fn(IndexId, String) -> IDX) -> SolarResult<Vec<IDX>> {
        let inner = Index::no_duplicate(inner, Self::NOUN)?
            .into_iter()
            .map(|s| build(Index::hash_id(&s), s))
            .collect::<Vec<_>>();

        Ok(inner)
    }
    
    fn inner(&self) -> &Vec<IDX>;
    fn inner_mut(&mut self) -> &mut Vec<IDX>;
    fn take_inner(&mut self) -> Vec<IDX>;
}

#[allow(private_bounds)]
pub trait Indexed<IDX: Idx>: IndexedInner<IDX> {
    fn get(&self, id: IndexId) -> SolarResult<&IDX> {
        self.inner().get(id as usize)
            .ok_or_else(|| SolarError::not_found(Self::NOUN))
    }
    
    fn get_keyed(&self, key: &str) -> SolarResult<&IDX> {
        self.inner().iter()
            .find(|idx| idx.key() == key)
            .ok_or_else(|| SolarError::not_found(Self::NOUN))
    }
    
    fn iter<'a>(&'a self) -> slice::Iter<'a, IDX> where IDX: 'a {
        self.inner().iter()
    }
    
    fn ids(&self) -> Vec<IndexId> { self.inner().iter().map(Idx::id).collect() }
    fn keys<'a>(&'a self) -> Vec<&'a str> where IDX: 'a { self.inner().iter().map(|i| i.key()).collect() }
    
    fn extend(&mut self, mut other: Self) -> SolarResult<()> {
        let right = other.keys();
        for key in self.keys() {
            if right.contains(&key) {
                return SolarError::err_duplicate(Self::NOUN, key);
            }
        }
        
        self.inner_mut().extend(other.take_inner());
        Ok(())
    }
}
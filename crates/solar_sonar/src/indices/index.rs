use crate::*;

pub type IndexId = u64;

pub trait Idx: Sized + Hash + Display {
    fn id(&self) -> IndexId;
    fn key(&self) -> &str;
}


#[derive(Debug, PartialEq, Eq)]
pub struct Index {
    characters: CharacterIndex,
    modes: ModeIndex,
    channel_names: ChannelNameIndex,
}

impl Index {
    pub fn new(modes: ModeIndex, characters: CharacterIndex, channel_names: ChannelNameIndex) -> Self {
        Self { modes, characters, channel_names}
    }
    
    pub fn hash_id(s: &str) -> IndexId {
        xxh3_64(s.as_bytes())
    }
    
    pub fn characters(&self) -> &CharacterIndex { &self.characters }
    pub fn modes(&self) -> &ModeIndex { &self.modes }
    pub fn chat_channels(&self) -> &ChannelNameIndex { &self.channel_names }
    
    pub fn chat_channels_mut(&mut self) -> &mut ChannelNameIndex {
        &mut self.channel_names
    }
    
    pub(crate) fn no_duplicate(v: Vec<String>, noun: ErrNoun) -> SolarResult<Vec<String>> {
        for s in &v {
            if v.iter().filter(|si| si.as_str() == s).count() > 1 {
                return SolarError::err_duplicate(noun, s);
            }
        }
        
        Ok(v)
    }
    
    pub(crate) fn no_duplicate_data<T: IndexKey>(data: &Vec<T>, noun: ErrNoun) -> SolarResult<Vec<(String, &T)>> {
        let mut out = Vec::with_capacity(data.len());
        for s in data {
            let key = s.index_key();
            if data.iter().filter(|si| si.index_key() == key).count() > 1 {
                return SolarError::err_duplicate(noun, key);
            }
            
            out.push((key.to_string(), s));
        }
        
        Ok(out)
    }
}

pub(crate) trait IndexedInner<IDX: Idx>: Sized {
    const NOUN: ErrNoun;
    
    fn new_inner(inner: Vec<String>, build: fn(IndexId, String) -> IDX) -> SolarResult<Vec<IDX>> {
        let inner = Index::no_duplicate(inner, Self::NOUN)?
            .into_iter()
            .map(|key| build(Index::hash_id(&key), key))
            .collect::<Vec<_>>();

        Ok(inner)
    }
    
    fn new_inner_from<T: IndexKey>(data: &Vec<T>, build: fn(IndexId, String, &T) -> IDX) -> SolarResult<Vec<IDX>> {
        let inner = Index::no_duplicate_data(data, Self::NOUN)?
            .into_iter()
            .map(|(key, data)| build(Index::hash_id(&key), key, data))
            .collect::<Vec<_>>();

        Ok(inner)
    }
    
    fn inner(&self) -> &Vec<IDX>;
    fn inner_mut(&mut self) -> &mut Vec<IDX>;
    fn take_inner(&mut self) -> Vec<IDX> { mem::take(self.inner_mut()) }
}

#[allow(private_bounds)]
pub trait Indexed<IDX: Idx>: IndexedInner<IDX> {
    fn get(&self, id: IndexId) -> SolarResult<&IDX> {
        self.inner().get(id as usize)
            .ok_or_else(|| SolarError::not_found(Self::NOUN, id))
    }
    
    fn find(&self, key: &str) -> SolarResult<&IDX> {
        self.inner().iter()
            .find(|idx| idx.key() == key)
            .ok_or_else(|| SolarError::not_found(Self::NOUN, key))
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

pub trait IndexKey {
    fn index_key(&self) -> &str;
}
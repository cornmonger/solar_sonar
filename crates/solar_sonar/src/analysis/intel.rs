use crate::*;

#[derive(
    Debug, Clone, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub struct IntelAnalysis {
    pub ambiguous: bool,
    pub systems: Vec<SolarId>,
    pub keywords: Vec<IntelKeyword>,
}

impl IntelAnalysis {
    pub fn in_danger(&self) -> bool {
        if self.systems.is_empty() {
            return false;
        }

        self.keywords.iter()
            .fold(None, |danger, word| match danger {
                None => Some(word.danger(self.ambiguous)),
                Some(last) => Some(last || word.danger(self.ambiguous)),
            })
            .unwrap_or_else(|| true)
    }
    
    pub fn system_ids(&self) -> &Vec<SolarId> {
       &self.systems
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub enum IntelKeyword {
    Clear,
    NoVisual,
    Status,
}

impl IntelKeyword {
    pub fn matches(s: &str) -> Option<Self> {
        match s {
            "CLEAR" | "CLR" => Some(Self::Clear),
            "NV" => Some(Self::NoVisual),
            "STATUS" | "STATUS?" | "CLR?" | "CLEAR?" => Some(Self::Status),
            _ => None
        }
    }

    pub fn danger(&self, has_unknown: bool) -> bool {
        match (self, has_unknown) {
            (Self::Clear, false) => false,
            (Self::Clear, true) => true,
            (Self::NoVisual, _) => true,
            (Self::Status, false) => false,
            (Self::Status, true) => true,
        }
    }
}


pub(crate) struct IntelAnalyzer;
impl IntelAnalyzer {
    pub(crate) fn analyze(&self, content: &str) -> IntelAnalysis {
        let mut keywords = vec![];
        let mut systems = vec![];
        let words = content.split_whitespace();
        let mut num_words = 0;
        for word in words {
            num_words += 1;
            let word = word.trim_end_matches('*').to_uppercase();
            if let Some(keyword) = IntelKeyword::matches(&word) {
                keywords.push(keyword);
            } else if let Some(system) = STAR_MAP.get_system_named(&word) {
                systems.push(system.id)
            }
        }
        
        let ambiguous = num_words > (systems.len() + keywords.len());
        
        IntelAnalysis {
            ambiguous,
            systems,
            keywords,
        }
    }
}

use crate::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub enum Callout {
    Opponent,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub enum CalloutKeyword {
    Nuet,
}

impl CalloutKeyword {
    pub(crate) fn matches_word(word: &str) -> Option<Self> {
        match word {
            "neut" | "neut!" | "nuet" | "nuet!" => Some(Self::Nuet),
            _ => None
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub struct CalloutAnalysis {
    pub call: Option<Callout>,
}

impl CalloutAnalysis {
    pub fn has_callout(&self) -> bool {
        self.call.is_some()
    }
    
    pub fn callout(&self) -> Option<&Callout> {
        self.call.as_ref()
    }
}

impl Analysis for CalloutAnalysis {
    fn dangerous(&self) -> bool {
        match self.call {
            Some(Callout::Opponent) => true,
            None => false,
        }
    }
}

pub(crate) struct CalloutAnalyzer;
impl Analyzer<CalloutAnalysis> for CalloutAnalyzer {
    fn analyze(self, content: &str) -> CalloutAnalysis {
        let words = content.split_whitespace().collect::<Vec<_>>();
        let mut callout = None;
        
        if let Some(first_word) = words.get(0) {
            if let Some(keyword) = CalloutKeyword::matches_word(&first_word.to_lowercase()) {
                callout = Some(match keyword {
                    CalloutKeyword::Nuet => Callout::Opponent,
                });
            }
        }
        
        CalloutAnalysis {
            call: callout,
        }
    }
}

use crate::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq,
    serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode
)]
pub enum AnalysisKind {
    Callout,
    Combat,
    Intel,
    Message,
}

impl AnalysisKind {
    pub fn try_from_input(s: &str) -> SolarResult<Self> {
        match s.to_lowercase().as_str() {
            "callout" => Ok(Self::Callout),
            "combat" => Ok(Self::Combat),
            "intel" => Ok(Self::Intel),
            "message" => Ok(Self::Message),
            _ => SolarError::err_enum(ErrNoun::AnalysisKind, s), 
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Hash, serde::Serialize, serde::Deserialize,
    bitcode::Encode, bitcode::Decode,
)]
pub struct LogAnalysis {
    pub(crate) intel: Option<IntelAnalysis>,
    pub(crate) callout: Option<CalloutAnalysis>,
}

impl LogAnalysis {
    pub fn intel(&self) -> Option<&IntelAnalysis> {
        self.intel.as_ref()
    }
    
    pub fn has_callout(&self) -> bool {
        self.callout.as_ref().is_some_and(|callout| callout.has_callout())
    }
    
    pub fn dangerous_callout(&self) -> bool {
        self.callout.as_ref().is_some_and(|callout| callout.dangerous())
    }

    pub fn callout(&self) -> Option<&Callout> {
        self.callout.as_ref().and_then(|callout| callout.callout())
    }
    
    pub fn system_ids(&self) -> &Vec<SolarId> {
        static EMPTY: Vec<SolarId> = vec![];
        
        if let Some(intel) = &self.intel {
            intel.system_ids()
        } else {
            &EMPTY
        }
    }
    
    pub fn has_single_system(&self) -> bool {
        self.intel.as_ref()
            .is_some_and(|intel| intel.systems.len() == 1)
    }
    
    pub fn dangerous(&self) -> bool {
        let mut danger = false;
        if let Some(intel) = &self.intel {
            danger = danger || intel.dangerous();
        }        
        if let Some(callout) = &self.callout {
            danger = danger || callout.dangerous();
        }
        
        danger
    }
    
    pub fn display_worthy(&self) -> bool {
        if self.dangerous() {
            true
        } else {
            false
        }
    }
}

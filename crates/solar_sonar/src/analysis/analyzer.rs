use crate::*;

pub trait Analysis {
    fn dangerous(&self) -> bool;
}

pub trait SystemContextAnalysis {
    fn system_ids(&self) -> &Vec<SolarId>;
}

pub(crate) trait Analyzer<A: Analysis> {
    fn analyze(self, content: &str) -> A;
}
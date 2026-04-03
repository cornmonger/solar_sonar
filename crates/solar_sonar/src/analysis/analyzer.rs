use crate::*;

pub trait Analysis {
    fn dangerous(&self) -> bool;
}

pub trait SystemContextAnalysis {
    fn system_ids(&self) -> &Vec<StarId>;
}

pub(crate) trait Analyzer<A: Analysis> {
    fn analyze(self, content: &str) -> A;
}
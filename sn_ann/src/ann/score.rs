use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct ScoredIdx {
    pub score: f32,
    pub idx: usize,
}

impl Eq for ScoredIdx {}

impl PartialEq for ScoredIdx {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}
impl PartialOrd for ScoredIdx {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}
impl Ord for ScoredIdx {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score
            .partial_cmp(&other.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[derive(Clone, Debug)]
pub struct MinScoredIdx(pub ScoredIdx);

impl Eq for MinScoredIdx {}
impl PartialEq for MinScoredIdx {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl PartialOrd for MinScoredIdx {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    } // invert
}
impl Ord for MinScoredIdx {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    } // invert
}

use crate::wfa2;

enum MemoryModel {
    MemoryHigh,
    MemoryMed,
    MemoryLow,
}

enum AlignmentScope {
    Score,
    Alignment,
}

enum AlignmentStatus {
    StatusSuccessful = wfa2::WF_STATUS_SUCCESSFUL as isize,
    StatusDropped = wfa2::WF_STATUS_HEURISTICALY_DROPPED as isize,
    StatusMaxScoreReached = wfa2::WF_STATUS_MAX_SCORE_REACHED as isize,
    StatusOOM = wfa2::WF_STATUS_OOM as isize,
}

struct WFAligner {
    attributes: wfa2::wavefront_aligner_attr_t,
}

impl WFAligner {
    pub fn new(alignment_scope: AlignmentScope, memory_model: MemoryModel) -> Self {
        let attributes = wfa2::wavefront_aligner_attr_default;
        Self { attributes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        let aligner = WFAligner::new(AlignmentScope::Alignment, MemoryModel::MemoryLow);
    }
}

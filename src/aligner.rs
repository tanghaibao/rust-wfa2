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
    attributes: *mut wfa2::wavefront_aligner_attr_t,
    wf_aligner: *mut wfa2::wavefront_aligner_t,
}

impl WFAligner {
    pub fn new(alignment_scope: AlignmentScope, memory_model: MemoryModel) -> Self {
        let attributes = unsafe { &mut wfa2::wavefront_aligner_attr_default };
        let wf_aligner = std::ptr::null_mut();
        Self {
            attributes,
            wf_aligner,
        }
    }
}

impl Drop for WFAligner {
    fn drop(&mut self) {
        unsafe {
            wfa2::wavefront_aligner_delete(self.wf_aligner);
        }
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

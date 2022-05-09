use crate::wfa2;
use std::ptr;
use std::slice;

pub enum MemoryModel {
    MemoryHigh,
    MemoryMed,
    MemoryLow,
}

pub enum AlignmentScope {
    Score,
    Alignment,
}

pub enum Heuristic {
    None,
    BandedStatic(i32, i32),
    BandedAdaptive(i32, i32, i32),
    WFadaptive(i32, i32, i32),
    XDrop(i32, i32),
    ZDrop(i32, i32),
}

#[derive(Debug, PartialEq, Eq)]
pub enum AlignmentStatus {
    StatusSuccessful = wfa2::WF_STATUS_SUCCESSFUL as isize,
    StatusDropped = wfa2::WF_STATUS_HEURISTICALY_DROPPED as isize,
    StatusMaxScoreReached = wfa2::WF_STATUS_MAX_SCORE_REACHED as isize,
    StatusOOM = wfa2::WF_STATUS_OOM as isize,
}

impl AlignmentStatus {
    fn from_i32(value: i32) -> Self {
        match value {
            x if x == wfa2::WF_STATUS_SUCCESSFUL as i32 => AlignmentStatus::StatusSuccessful,
            wfa2::WF_STATUS_HEURISTICALY_DROPPED => AlignmentStatus::StatusDropped,
            wfa2::WF_STATUS_MAX_SCORE_REACHED => AlignmentStatus::StatusMaxScoreReached,
            wfa2::WF_STATUS_OOM => AlignmentStatus::StatusOOM,
            _ => panic!("Unknown alignment status: {}", value),
        }
    }
}

struct WFAttributes {
    inner: wfa2::wavefront_aligner_attr_t,
}

impl WFAttributes {
    fn default() -> Self {
        Self {
            inner: unsafe { wfa2::wavefront_aligner_attr_default },
        }
    }

    fn memory_model(mut self, memory_model: MemoryModel) -> Self {
        let memory_mode = match memory_model {
            MemoryModel::MemoryHigh => wfa2::wavefront_memory_t_wavefront_memory_high,
            MemoryModel::MemoryMed => wfa2::wavefront_memory_t_wavefront_memory_med,
            MemoryModel::MemoryLow => wfa2::wavefront_memory_t_wavefront_memory_low,
        };
        self.inner.memory_mode = memory_mode;
        self
    }

    fn alignment_scope(mut self, alignment_scope: AlignmentScope) -> Self {
        let alignment_scope = match alignment_scope {
            AlignmentScope::Score => wfa2::alignment_scope_t_compute_score,
            AlignmentScope::Alignment => wfa2::alignment_scope_t_compute_alignment,
        };
        self.inner.alignment_scope = alignment_scope;
        self
    }

    fn affine_penalties(
        mut self,
        match_: i32,
        mismatch: i32,
        gap_opening: i32,
        gap_extension: i32,
    ) -> Self {
        self.inner.affine_penalties.match_ = match_;
        self.inner.affine_penalties.mismatch = mismatch;
        self.inner.affine_penalties.gap_opening = gap_opening;
        self.inner.affine_penalties.gap_extension = gap_extension;
        self
    }
}

struct WFAligner {
    attributes: WFAttributes,
    inner: *mut wfa2::wavefront_aligner_t,
}

impl WFAligner {
    pub fn new(alignment_scope: AlignmentScope, memory_model: MemoryModel) -> Self {
        let attributes = WFAttributes::default()
            .memory_model(memory_model)
            .alignment_scope(alignment_scope);
        Self {
            attributes,
            inner: ptr::null_mut(),
        }
    }
}

impl Drop for WFAligner {
    fn drop(&mut self) {
        unsafe {
            if !self.inner.is_null() {
                wfa2::wavefront_aligner_delete(self.inner);
            }
        }
    }
}

pub trait Align {
    fn aligner_inner(&self) -> *mut wfa2::wavefront_aligner_t;

    fn align_end_to_end(&mut self, pattern: &[u8], text: &[u8]) -> AlignmentStatus {
        let status = unsafe {
            // Configure
            wfa2::wavefront_aligner_set_alignment_end_to_end(self.aligner_inner());
            // Align
            wfa2::wavefront_align(
                self.aligner_inner(),
                pattern.as_ptr() as *const i8,
                pattern.len() as i32,
                text.as_ptr() as *const i8,
                text.len() as i32,
            )
        };
        AlignmentStatus::from_i32(status)
    }

    fn alignment_score(&self) -> i32 {
        unsafe { (*self.aligner_inner()).cigar.score }
    }

    fn alignment_cigar(&self) -> String {
        let cigar_str = unsafe {
            let begin_offset = (*self.aligner_inner()).cigar.begin_offset;
            let cigar_operations = (*self.aligner_inner())
                .cigar
                .operations
                .offset(begin_offset as isize) as *const u8;
            let cigar_length = ((*self.aligner_inner()).cigar.end_offset - begin_offset) as usize;
            slice::from_raw_parts(cigar_operations, cigar_length)
        };
        String::from_utf8_lossy(cigar_str).to_string()
    }

    fn alignment_matching(&self, pattern: &[u8], text: &[u8]) -> (String, String, String) {
        let cigar = self.alignment_cigar();
        let mut pattern_iter = pattern.iter();
        let mut text_iter = text.iter();
        let mut pattern_match = String::new();
        let mut middle_match = String::new();
        let mut text_match = String::new();
        for c in cigar.chars() {
            match c {
                'M' | 'X' => {
                    pattern_match.push(*pattern_iter.next().unwrap() as char);
                    middle_match.push('|');
                    text_match.push(*text_iter.next().unwrap() as char);
                }
                'D' => {
                    pattern_match.push(*pattern_iter.next().unwrap() as char);
                    middle_match.push(' ');
                    text_match.push('-');
                }
                'I' => {
                    pattern_match.push('-');
                    middle_match.push(' ');
                    text_match.push(*text_iter.next().unwrap() as char);
                }
                _ => panic!("Unknown cigar operation: {}", c),
            }
        }
        (pattern_match, middle_match, text_match)
    }

    fn set_heuristic(&mut self, heuristic: Heuristic) {
        unsafe {
            match heuristic {
                Heuristic::None => wfa2::wavefront_aligner_set_heuristic_none(self.aligner_inner()),
                Heuristic::BandedStatic(band_min_k, band_max_k) => {
                    wfa2::wavefront_aligner_set_heuristic_banded_static(
                        self.aligner_inner(),
                        band_min_k,
                        band_max_k,
                    )
                }
                Heuristic::BandedAdaptive(band_min_k, band_max_k, steps_between_cutoffs) => {
                    wfa2::wavefront_aligner_set_heuristic_banded_adaptive(
                        self.aligner_inner(),
                        band_min_k,
                        band_max_k,
                        steps_between_cutoffs,
                    )
                }
                Heuristic::WFadaptive(
                    min_wavefront_length,
                    max_distance_threshold,
                    steps_between_cutoffs,
                ) => wfa2::wavefront_aligner_set_heuristic_wfadaptive(
                    self.aligner_inner(),
                    min_wavefront_length,
                    max_distance_threshold,
                    steps_between_cutoffs,
                ),
                Heuristic::XDrop(xdrop, steps_between_cutoffs) => {
                    wfa2::wavefront_aligner_set_heuristic_xdrop(
                        self.aligner_inner(),
                        xdrop,
                        steps_between_cutoffs,
                    )
                }
                Heuristic::ZDrop(zdrop, steps_between_cutoffs) => {
                    wfa2::wavefront_aligner_set_heuristic_zdrop(
                        self.aligner_inner(),
                        zdrop,
                        steps_between_cutoffs,
                    )
                }
            }
        }
    }
}

/// Gap-Affine Aligner (a.k.a Smith-Waterman-Gotoh)
pub struct WFAlignerGapAffine {
    aligner: WFAligner,
}

impl WFAlignerGapAffine {
    pub fn new(
        mismatch: i32,
        gap_opening: i32,
        gap_extension: i32,
        alignment_scope: AlignmentScope,
        memory_model: MemoryModel,
    ) -> Self {
        let mut aligner = WFAligner::new(alignment_scope, memory_model);
        aligner.attributes =
            WFAttributes::default().affine_penalties(0, mismatch, gap_opening, gap_extension);
        unsafe {
            aligner.inner = wfa2::wavefront_aligner_new(&mut aligner.attributes.inner);
        }
        Self { aligner }
    }
}

impl Align for WFAlignerGapAffine {
    fn aligner_inner(&self) -> *mut wfa2::wavefront_aligner_t {
        self.aligner.inner
    }
}

pub struct WFAlignerIndel {
    aligner: WFAligner,
}

/// Indel Aligner (a.k.a Longest Common Subsequence - LCS)
impl WFAlignerIndel {
    pub fn new(alignment_scope: AlignmentScope, memory_model: MemoryModel) -> Self {
        let mut aligner = WFAligner::new(alignment_scope, memory_model);
        aligner.attributes.inner.distance_metric = wfa2::distance_metric_t_indel;
        unsafe {
            aligner.inner = wfa2::wavefront_aligner_new(&mut aligner.attributes.inner);
        }
        Self { aligner }
    }
}

impl Align for WFAlignerIndel {
    fn aligner_inner(&self) -> *mut wfa2::wavefront_aligner_t {
        self.aligner.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_aligner() -> impl Align {
        test_aligner_gap_affine()
    }

    fn test_aligner_gap_affine() -> WFAlignerGapAffine {
        WFAlignerGapAffine::new(4, 6, 2, AlignmentScope::Alignment, MemoryModel::MemoryLow)
    }

    #[test]
    fn test_aligner_indel() {
        let mut aligner = WFAlignerIndel::new(AlignmentScope::Alignment, MemoryModel::MemoryHigh);
        let pattern = b"TCTTTACTCGCGCGTTGGAGAAATACAATAGT";
        let text = b"TCTATACTGCGCGTTTGGAGAAATAAAATAGT";
        let status = aligner.align_end_to_end(pattern, text);
        assert_eq!(status, AlignmentStatus::StatusSuccessful);
    }

    #[test]
    fn test_align_end_to_end() {
        let mut aligner = test_aligner();
        let pattern = b"TCTTTACTCGCGCGTTGGAGAAATACAATAGT";
        let text = b"TCTATACTGCGCGTTTGGAGAAATAAAATAGT";
        let status = aligner.align_end_to_end(pattern, text);
        assert_eq!(status, AlignmentStatus::StatusSuccessful);
        assert_eq!(aligner.alignment_score(), -24);
        assert_eq!(
            aligner.alignment_cigar(),
            "MMMXMMMMDMMMMMMMIMMMMMMMMMXMMMMMM"
        );
        let (a, b, c) = aligner.alignment_matching(pattern, text);
        assert_eq!(
            format!("{}\n{}\n{}", a, b, c),
            "TCTTTACTCGCGCGTT-GGAGAAATACAATAGT\n|||||||| ||||||| ||||||||||||||||\nTCTATACT-GCGCGTTTGGAGAAATAAAATAGT"
        );
    }

    #[test]
    fn test_set_heuristic() {
        let mut aligner = test_aligner();
        aligner.set_heuristic(Heuristic::BandedStatic(1, 2));
        aligner.set_heuristic(Heuristic::BandedAdaptive(1, 2, 3));
        aligner.set_heuristic(Heuristic::WFadaptive(1, 2, 3));
        aligner.set_heuristic(Heuristic::XDrop(1, 2));
        aligner.set_heuristic(Heuristic::ZDrop(1, 2));
    }
}

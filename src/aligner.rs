use crate::wfa2;
use std::slice;

enum MemoryModel {
    MemoryHigh,
    MemoryMed,
    MemoryLow,
}

enum AlignmentScope {
    Score,
    Alignment,
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

struct WFAligner {
    attributes: *mut wfa2::wavefront_aligner_attr_t,
    inner: *mut wfa2::wavefront_aligner_t,
}

impl WFAligner {
    pub fn new(alignment_scope: AlignmentScope, memory_model: MemoryModel) -> Self {
        let attributes = unsafe {
            let attrib = &mut wfa2::wavefront_aligner_attr_default;
            attrib.memory_mode = match memory_model {
                MemoryModel::MemoryHigh => wfa2::wavefront_memory_t_wavefront_memory_high,
                MemoryModel::MemoryMed => wfa2::wavefront_memory_t_wavefront_memory_med,
                MemoryModel::MemoryLow => wfa2::wavefront_memory_t_wavefront_memory_low,
            };
            attrib.alignment_scope = match alignment_scope {
                AlignmentScope::Score => wfa2::alignment_scope_t_compute_score,
                AlignmentScope::Alignment => wfa2::alignment_scope_t_compute_alignment,
            };
            attrib
        };
        Self {
            attributes,
            inner: std::ptr::null_mut(),
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

struct WFAlignerGapAffine {
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
        unsafe {
            (*aligner.attributes).affine_penalties.match_ = 0;
            (*aligner.attributes).affine_penalties.mismatch = mismatch;
            (*aligner.attributes).affine_penalties.gap_opening = gap_opening;
            (*aligner.attributes).affine_penalties.gap_extension = gap_extension;
            aligner.inner = wfa2::wavefront_aligner_new(aligner.attributes);
        }
        Self { aligner }
    }
}

pub trait Align {
    fn align_end_to_end(&mut self, pattern: &[u8], text: &[u8]) -> AlignmentStatus;

    fn alignment_score(&self) -> i32;

    fn alignment_cigar(&self) -> String;

    fn alignment_matching(&self, pattern: &[u8], text: &[u8]) -> (String, String, String);
}

impl Align for WFAlignerGapAffine {
    fn align_end_to_end(&mut self, pattern: &[u8], text: &[u8]) -> AlignmentStatus {
        unsafe {
            // Configure
            wfa2::wavefront_aligner_set_alignment_end_to_end(self.aligner.inner);
            // Align
            let status = wfa2::wavefront_align(
                self.aligner.inner,
                pattern.as_ptr() as *const i8,
                pattern.len() as i32,
                text.as_ptr() as *const i8,
                text.len() as i32,
            );
            AlignmentStatus::from_i32(status)
        }
    }

    fn alignment_score(&self) -> i32 {
        unsafe { (*self.aligner.inner).cigar.score }
    }

    fn alignment_cigar(&self) -> String {
        let cigar_str = unsafe {
            let begin_offset = (*self.aligner.inner).cigar.begin_offset;
            let cigar_operations = (*self.aligner.inner)
                .cigar
                .operations
                .offset(begin_offset as isize) as *const u8;
            let cigar_length = ((*self.aligner.inner).cigar.end_offset - begin_offset) as usize;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_end_to_end() {
        let mut aligner =
            WFAlignerGapAffine::new(4, 6, 2, AlignmentScope::Alignment, MemoryModel::MemoryLow);
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
}

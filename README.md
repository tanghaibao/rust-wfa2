# Rust bindings for WFA2-Lib

Rust language bindgings for the excellent
[WFA2-Lib](https://github.com/smarco/WFA2-lib) library.

Work in progress. Tests and features are not yet complete.

## Usage

```rust
let mut aligner = aligner_gap_affine();
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
```
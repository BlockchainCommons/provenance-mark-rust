use std::collections::{HashMap, HashSet};

use crate::ProvenanceMark;

/// Issue flagged during validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationIssue {
    /// Exact duplicate mark found
    ExactDuplicate,
    /// Hash mismatch between consecutive marks
    HashMismatch { expected: Vec<u8>, actual: Vec<u8> },
    /// Key mismatch between consecutive marks
    KeyMismatch,
    /// Sequence number gap
    SequenceGap { expected: u32, actual: u32 },
    /// Date ordering violation
    DateOrdering {
        previous: dcbor::Date,
        next: dcbor::Date,
    },
    /// Non-genesis mark at sequence 0
    NonGenesisAtZero,
    /// Invalid genesis key
    InvalidGenesisKey,
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationIssue::ExactDuplicate => {
                write!(f, "exact duplicate mark found")
            }
            ValidationIssue::HashMismatch { expected, actual } => {
                write!(
                    f,
                    "hash mismatch: expected {}, got {}",
                    hex::encode(expected),
                    hex::encode(actual)
                )
            }
            ValidationIssue::KeyMismatch => {
                write!(
                    f,
                    "key mismatch: current hash was not generated from next key"
                )
            }
            ValidationIssue::SequenceGap { expected, actual } => {
                write!(
                    f,
                    "sequence number gap: expected {}, got {}",
                    expected, actual
                )
            }
            ValidationIssue::DateOrdering { previous, next } => {
                write!(
                    f,
                    "date must be equal or later: previous is {}, next is {}",
                    previous, next
                )
            }
            ValidationIssue::NonGenesisAtZero => {
                write!(f, "non-genesis mark at sequence 0")
            }
            ValidationIssue::InvalidGenesisKey => {
                write!(f, "genesis mark must have key equal to chain_id")
            }
        }
    }
}

impl std::error::Error for ValidationIssue {}

/// A mark with any issues flagged during validation
#[derive(Debug, Clone)]
pub struct FlaggedMark {
    mark: ProvenanceMark,
    issues: Vec<ValidationIssue>,
}

impl FlaggedMark {
    fn new(mark: ProvenanceMark) -> Self { Self { mark, issues: Vec::new() } }

    fn with_issue(mark: ProvenanceMark, issue: ValidationIssue) -> Self {
        Self { mark, issues: vec![issue] }
    }

    pub fn mark(&self) -> &ProvenanceMark { &self.mark }
    pub fn issues(&self) -> &[ValidationIssue] { &self.issues }
}

/// Report for a contiguous sequence of marks within a chain
#[derive(Debug, Clone)]
pub struct SequenceReport {
    start_seq: u32,
    end_seq: u32,
    marks: Vec<FlaggedMark>,
}

impl SequenceReport {
    pub fn start_seq(&self) -> u32 { self.start_seq }
    pub fn end_seq(&self) -> u32 { self.end_seq }
    pub fn marks(&self) -> &[FlaggedMark] { &self.marks }
}

/// Report for a chain of marks with the same chain ID
#[derive(Debug, Clone)]
pub struct ChainReport {
    chain_id: Vec<u8>,
    has_genesis: bool,
    marks: Vec<ProvenanceMark>,
    sequences: Vec<SequenceReport>,
}

impl ChainReport {
    pub fn chain_id(&self) -> &[u8] { &self.chain_id }
    pub fn has_genesis(&self) -> bool { self.has_genesis }
    pub fn marks(&self) -> &[ProvenanceMark] { &self.marks }
    pub fn sequences(&self) -> &[SequenceReport] { &self.sequences }

    /// Get the chain ID as a hex string for display
    pub fn chain_id_hex(&self) -> String { hex::encode(&self.chain_id) }
}

/// Complete validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    original_marks: Vec<ProvenanceMark>,
    deduplicated_marks: Vec<ProvenanceMark>,
    chains: Vec<ChainReport>,
}

impl ValidationReport {
    pub fn original_marks(&self) -> &[ProvenanceMark] { &self.original_marks }
    pub fn deduplicated_marks(&self) -> &[ProvenanceMark] { &self.deduplicated_marks }
    pub fn chains(&self) -> &[ChainReport] { &self.chains }

    /// Validate a collection of provenance marks
    pub fn validate(marks: Vec<ProvenanceMark>) -> Self {
        let original_marks = marks.clone();

        // Deduplicate exact duplicates
        let mut seen = HashSet::new();
        let mut deduplicated_marks = Vec::new();
        for mark in marks {
            if seen.insert(mark.clone()) {
                deduplicated_marks.push(mark);
            }
        }

        // Bin marks by chain ID
        let mut chain_bins: HashMap<Vec<u8>, Vec<ProvenanceMark>> =
            HashMap::new();
        for mark in &deduplicated_marks {
            chain_bins
                .entry(mark.chain_id().to_vec())
                .or_default()
                .push(mark.clone());
        }

        // Process each chain
        let mut chains = Vec::new();
        for (chain_id_bytes, mut chain_marks) in chain_bins {
            // Sort by sequence number
            chain_marks.sort_by_key(|m| m.seq());

            // Check for genesis mark
            let has_genesis = chain_marks
                .first()
                .is_some_and(|m| m.seq() == 0 && m.is_genesis());

            // Build sequence bins
            let sequences = Self::build_sequence_bins(&chain_marks);

            chains.push(ChainReport {
                chain_id: chain_id_bytes,
                has_genesis,
                marks: chain_marks,
                sequences,
            });
        }

        // Sort chains by chain ID for consistent output
        chains.sort_by(|a, b| a.chain_id.cmp(&b.chain_id));

        ValidationReport { original_marks, deduplicated_marks, chains }
    }

    fn build_sequence_bins(marks: &[ProvenanceMark]) -> Vec<SequenceReport> {
        let mut sequences = Vec::new();
        let mut current_sequence: Vec<FlaggedMark> = Vec::new();

        for (i, mark) in marks.iter().enumerate() {
            if i == 0 {
                // First mark starts a sequence
                current_sequence.push(FlaggedMark::new(mark.clone()));
            } else {
                let prev = &marks[i - 1];

                // Check if this mark follows the previous one
                match prev.precedes_opt(mark) {
                    Ok(()) => {
                        // Continues the current sequence
                        current_sequence.push(FlaggedMark::new(mark.clone()));
                    }
                    Err(e) => {
                        // Breaks the sequence - save current and start new
                        if !current_sequence.is_empty() {
                            sequences.push(Self::create_sequence_report(
                                current_sequence,
                            ));
                        }

                        // Start new sequence with this mark, flagged with the
                        // issue
                        let issue = match e {
                            crate::Error::Validation(v) => v,
                            _ => ValidationIssue::KeyMismatch, // Fallback
                        };
                        current_sequence =
                            vec![FlaggedMark::with_issue(mark.clone(), issue)];
                    }
                }
            }
        }

        // Add the final sequence
        if !current_sequence.is_empty() {
            sequences.push(Self::create_sequence_report(current_sequence));
        }

        sequences
    }

    fn create_sequence_report(marks: Vec<FlaggedMark>) -> SequenceReport {
        let start_seq = marks.first().map(|m| m.mark.seq()).unwrap_or(0);
        let end_seq = marks.last().map(|m| m.mark.seq()).unwrap_or(0);

        SequenceReport { start_seq, end_seq, marks }
    }
}

impl ProvenanceMark {
    /// Validate a collection of provenance marks
    ///
    /// This method analyzes the provided marks and produces a comprehensive
    /// validation report that includes:
    /// - Deduplication of exact duplicates
    /// - Organization by chain ID
    /// - Detection of genesis marks
    /// - Identification of contiguous sequences
    /// - Flagging of validation issues (hash mismatches, sequence gaps, etc.)
    pub fn validate(marks: Vec<ProvenanceMark>) -> ValidationReport {
        ValidationReport::validate(marks)
    }
}

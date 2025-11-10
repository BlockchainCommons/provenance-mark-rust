use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::ProvenanceMark;

// Helper module for serializing ProvenanceMark as UR string
mod provenance_mark_as_ur {
    use bc_ur::UREncodable;
    use serde::Serializer;

    use crate::ProvenanceMark;

    pub fn serialize<S>(
        mark: &ProvenanceMark,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&mark.ur_string())
    }
}

// Helper module for serializing Vec<ProvenanceMark> as Vec<UR string>
mod provenance_marks_as_ur {
    use bc_ur::UREncodable;
    use serde::Serializer;

    use crate::ProvenanceMark;

    pub fn serialize<S>(
        marks: &[ProvenanceMark],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(marks.len()))?;
        for mark in marks {
            seq.serialize_element(&mark.ur_string())?;
        }
        seq.end()
    }
}

// Helper module for serializing dcbor::Date as ISO8601 string
mod date_as_iso8601 {
    use serde::Serializer;

    pub fn serialize<S>(
        date: &dcbor::Date,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&date.to_string())
    }
}

/// Issue flagged during validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ValidationIssue {
    /// Hash mismatch between consecutive marks
    HashMismatch {
        #[serde(with = "hex")]
        expected: Vec<u8>,
        #[serde(with = "hex")]
        actual: Vec<u8>,
    },
    /// Key mismatch between consecutive marks
    KeyMismatch,
    /// Sequence number gap
    SequenceGap { expected: u32, actual: u32 },
    /// Date ordering violation
    DateOrdering {
        #[serde(serialize_with = "date_as_iso8601::serialize")]
        previous: dcbor::Date,
        #[serde(serialize_with = "date_as_iso8601::serialize")]
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
#[derive(Debug, Clone, Serialize)]
pub struct FlaggedMark {
    #[serde(serialize_with = "provenance_mark_as_ur::serialize")]
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
#[derive(Debug, Clone, Serialize)]
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
#[derive(Debug, Clone, Serialize)]
pub struct ChainReport {
    #[serde(with = "hex")]
    chain_id: Vec<u8>,
    has_genesis: bool,
    #[serde(serialize_with = "provenance_marks_as_ur::serialize")]
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
#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    #[serde(serialize_with = "provenance_marks_as_ur::serialize")]
    marks: Vec<ProvenanceMark>,
    chains: Vec<ChainReport>,
}

impl ValidationReport {
    pub fn marks(&self) -> &[ProvenanceMark] { &self.marks }
    pub fn chains(&self) -> &[ChainReport] { &self.chains }

    /// Format the validation report as human-readable text.
    ///
    /// Returns a formatted string if the report contains interesting
    /// information (issues, multiple chains, or multiple sequences).
    /// Returns an empty string if the report represents a single perfect chain
    /// with no issues.
    pub fn format(&self) -> String {
        if !self.is_interesting() {
            return String::new();
        }

        let mut lines = Vec::new();

        // Report summary
        lines.push(format!("Total marks: {}", self.marks.len()));
        lines.push(format!("Chains: {}", self.chains.len()));
        lines.push(String::new());

        // Report each chain
        for (chain_idx, chain) in self.chains.iter().enumerate() {
            // Show short chain ID (first 4 bytes)
            let chain_id_hex = chain.chain_id_hex();
            let short_chain_id = if chain_id_hex.len() > 8 {
                &chain_id_hex[..8]
            } else {
                &chain_id_hex
            };

            lines.push(format!("Chain {}: {}", chain_idx + 1, short_chain_id));

            if !chain.has_genesis() {
                lines.push("  Warning: No genesis mark found".to_string());
            }

            // Report each sequence
            for seq in chain.sequences() {
                // Report each mark in the sequence
                for flagged_mark in seq.marks() {
                    let mark = flagged_mark.mark();
                    let short_id = mark.identifier();
                    let seq_num = mark.seq();

                    // Build the mark line with annotations
                    let mut annotations = Vec::new();

                    // Check if it's genesis
                    if mark.is_genesis() {
                        annotations.push("genesis mark".to_string());
                    }

                    // Add issue annotations
                    for issue in flagged_mark.issues() {
                        let issue_str = match issue {
                            ValidationIssue::SequenceGap {
                                expected,
                                actual: _,
                            } => {
                                format!("gap: {} missing", expected)
                            }
                            ValidationIssue::DateOrdering {
                                previous,
                                next,
                            } => {
                                format!("date {} < {}", previous, next)
                            }
                            ValidationIssue::HashMismatch { .. } => {
                                "hash mismatch".to_string()
                            }
                            ValidationIssue::KeyMismatch => {
                                "key mismatch".to_string()
                            }
                            ValidationIssue::NonGenesisAtZero => {
                                "non-genesis at seq 0".to_string()
                            }
                            ValidationIssue::InvalidGenesisKey => {
                                "invalid genesis key".to_string()
                            }
                        };
                        annotations.push(issue_str);
                    }

                    // Format the line
                    if annotations.is_empty() {
                        lines.push(format!("  {}: {}", seq_num, short_id));
                    } else {
                        lines.push(format!(
                            "  {}: {} ({})",
                            seq_num,
                            short_id,
                            annotations.join(", ")
                        ));
                    }
                }
            }

            lines.push(String::new());
        }

        lines.join("\n").trim_end().to_string()
    }

    /// Check if the validation report contains interesting information.
    ///
    /// Returns false for a single perfect chain with no issues, true otherwise.
    fn is_interesting(&self) -> bool {
        // Not interesting if empty
        if self.chains.is_empty() {
            return false;
        }

        // Check if any chain is missing genesis
        for chain in &self.chains {
            if !chain.has_genesis() {
                return true;
            }
        }

        // Not interesting if single chain with single perfect sequence
        if self.chains.len() == 1 {
            let chain = &self.chains[0];
            if chain.sequences().len() == 1 {
                let seq = &chain.sequences()[0];
                // Check if the sequence has no issues
                if seq.marks().iter().all(|m| m.issues().is_empty()) {
                    return false;
                }
            }
        }

        true
    }

    /// Check if the validation report has any issues.
    ///
    /// Returns true if there are validation issues, missing genesis,
    /// multiple chains, or multiple sequences.
    pub fn has_issues(&self) -> bool {
        // Missing genesis is considered an issue
        for chain in &self.chains {
            if !chain.has_genesis() {
                return true;
            }
        }

        // Check for validation issues in marks
        for chain in &self.chains {
            for seq in chain.sequences() {
                for mark in seq.marks() {
                    if !mark.issues().is_empty() {
                        return true;
                    }
                }
            }
        }

        // Multiple chains or sequences are also considered issues
        if self.chains.len() > 1 {
            return true;
        }

        if self.chains.len() == 1 && self.chains[0].sequences().len() > 1 {
            return true;
        }

        false
    }

    /// Validate a collection of provenance marks
    /// Validate a collection of provenance marks
    pub fn validate(marks: Vec<ProvenanceMark>) -> Self {
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

        ValidationReport { marks: deduplicated_marks, chains }
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

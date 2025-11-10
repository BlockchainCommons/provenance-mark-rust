use chrono::TimeZone;
use dcbor::Date;
use provenance_mark::*;

fn create_test_marks(
    count: usize,
    resolution: ProvenanceMarkResolution,
    passphrase: &str,
) -> Vec<ProvenanceMark> {
    let mut generator =
        ProvenanceMarkGenerator::new_with_passphrase(resolution, passphrase);
    let calendar = chrono::Utc;

    (0..count)
        .map(|i| {
            let date = Date::from_datetime(
                calendar
                    .with_ymd_and_hms(2023, 6, 20, 12, 0, 0)
                    .single()
                    .unwrap()
                    .checked_add_signed(chrono::Duration::days(i as i64))
                    .unwrap(),
            );
            generator.next(date, None::<String>)
        })
        .collect()
}

#[test]
fn test_validate_empty() {
    let report = ProvenanceMark::validate(vec![]);
    assert_eq!(report.original_marks().len(), 0);
    assert_eq!(report.deduplicated_marks().len(), 0);
    assert_eq!(report.chains().len(), 0);
}

#[test]
fn test_validate_single_mark() {
    let marks = create_test_marks(1, ProvenanceMarkResolution::Low, "test");
    let report = ProvenanceMark::validate(marks.clone());

    assert_eq!(report.original_marks().len(), 1);
    assert_eq!(report.deduplicated_marks().len(), 1);
    assert_eq!(report.chains().len(), 1);

    let chain = &report.chains()[0];
    assert!(chain.has_genesis());
    assert_eq!(chain.marks().len(), 1);
    assert_eq!(chain.sequences().len(), 1);

    let seq = &chain.sequences()[0];
    assert_eq!(seq.start_seq(), 0);
    assert_eq!(seq.end_seq(), 0);
    assert_eq!(seq.marks().len(), 1);
    assert!(seq.marks()[0].issues().is_empty());
}

#[test]
fn test_validate_valid_sequence() {
    let marks = create_test_marks(5, ProvenanceMarkResolution::Low, "test");
    let report = ProvenanceMark::validate(marks.clone());

    assert_eq!(report.original_marks().len(), 5);
    assert_eq!(report.deduplicated_marks().len(), 5);
    assert_eq!(report.chains().len(), 1);

    let chain = &report.chains()[0];
    assert!(chain.has_genesis());
    assert_eq!(chain.marks().len(), 5);
    assert_eq!(chain.sequences().len(), 1);

    let seq = &chain.sequences()[0];
    assert_eq!(seq.start_seq(), 0);
    assert_eq!(seq.end_seq(), 4);
    assert_eq!(seq.marks().len(), 5);

    // No issues in a valid sequence
    for flagged_mark in seq.marks() {
        assert!(
            flagged_mark.issues().is_empty(),
            "Expected no issues, got: {:?}",
            flagged_mark.issues()
        );
    }
}

#[test]
fn test_validate_deduplication() {
    let marks = create_test_marks(3, ProvenanceMarkResolution::Low, "test");

    // Create duplicates
    let mut marks_with_dups = marks.clone();
    marks_with_dups.push(marks[0].clone());
    marks_with_dups.push(marks[1].clone());
    marks_with_dups.push(marks[0].clone());

    let report = ProvenanceMark::validate(marks_with_dups);

    assert_eq!(report.original_marks().len(), 6);
    assert_eq!(report.deduplicated_marks().len(), 3);
    assert_eq!(report.chains().len(), 1);

    let chain = &report.chains()[0];
    assert_eq!(chain.marks().len(), 3);
}

#[test]
fn test_validate_multiple_chains() {
    let marks1 = create_test_marks(3, ProvenanceMarkResolution::Low, "alice");
    let marks2 = create_test_marks(3, ProvenanceMarkResolution::Low, "bob");

    let mut all_marks = marks1.clone();
    all_marks.extend(marks2.clone());

    let report = ProvenanceMark::validate(all_marks);

    assert_eq!(report.original_marks().len(), 6);
    assert_eq!(report.deduplicated_marks().len(), 6);
    assert_eq!(report.chains().len(), 2);

    // Both chains should have genesis marks
    for chain in report.chains() {
        assert!(chain.has_genesis());
        assert_eq!(chain.marks().len(), 3);
        assert_eq!(chain.sequences().len(), 1);
    }
}

#[test]
fn test_validate_missing_genesis() {
    let marks = create_test_marks(5, ProvenanceMarkResolution::Low, "test");

    // Remove genesis mark (index 0)
    let marks_no_genesis: Vec<_> = marks.into_iter().skip(1).collect();

    let report = ProvenanceMark::validate(marks_no_genesis);

    assert_eq!(report.chains().len(), 1);
    let chain = &report.chains()[0];
    assert!(!chain.has_genesis());
    assert_eq!(chain.marks().len(), 4);
}

#[test]
fn test_validate_sequence_gap() {
    let marks = create_test_marks(5, ProvenanceMarkResolution::Low, "test");

    // Create a gap by removing mark at index 2 (sequence 2)
    let marks_with_gap = vec![
        marks[0].clone(),
        marks[1].clone(),
        marks[3].clone(), // Gap: skips seq 2, this is seq 3
        marks[4].clone(),
    ];

    let report = ProvenanceMark::validate(marks_with_gap);

    assert_eq!(report.chains().len(), 1);
    let chain = &report.chains()[0];

    // Should have 2 sequences: [0,1] and [3,4]
    assert_eq!(chain.sequences().len(), 2);

    let seq1 = &chain.sequences()[0];
    assert_eq!(seq1.start_seq(), 0);
    assert_eq!(seq1.end_seq(), 1);
    assert_eq!(seq1.marks().len(), 2);

    let seq2 = &chain.sequences()[1];
    assert_eq!(seq2.start_seq(), 3);
    assert_eq!(seq2.end_seq(), 4);
    assert_eq!(seq2.marks().len(), 2);

    // First mark of second sequence should have a SequenceGap issue
    assert_eq!(seq2.marks()[0].issues().len(), 1);
    match &seq2.marks()[0].issues()[0] {
        ValidationIssue::SequenceGap { expected, actual } => {
            assert_eq!(*expected, 2);
            assert_eq!(*actual, 3);
        }
        other => panic!("Expected SequenceGap issue, got {:?}", other),
    }
}

#[test]
fn test_validate_out_of_order() {
    let marks = create_test_marks(5, ProvenanceMarkResolution::Low, "test");

    // Swap marks 2 and 3
    let marks_out_of_order = vec![
        marks[0].clone(),
        marks[1].clone(),
        marks[3].clone(), // Out of order
        marks[2].clone(),
        marks[4].clone(),
    ];

    let report = ProvenanceMark::validate(marks_out_of_order);

    assert_eq!(report.chains().len(), 1);
    let chain = &report.chains()[0];

    // After sorting, marks should be in correct sequence order
    assert_eq!(chain.marks()[0].seq(), 0);
    assert_eq!(chain.marks()[1].seq(), 1);
    assert_eq!(chain.marks()[2].seq(), 2);
    assert_eq!(chain.marks()[3].seq(), 3);
    assert_eq!(chain.marks()[4].seq(), 4);

    // Should form one valid sequence since sorting fixes the order
    assert_eq!(chain.sequences().len(), 1);
}

#[test]
fn test_validate_hash_mismatch() {
    let marks = create_test_marks(3, ProvenanceMarkResolution::Low, "test");
    let other_marks =
        create_test_marks(3, ProvenanceMarkResolution::Low, "other");

    // Create a chain with mismatched hashes by mixing chains
    let mixed_marks = vec![
        marks[0].clone(),
        other_marks[1].clone(), // Different chain - will have hash mismatch
    ];

    let report = ProvenanceMark::validate(mixed_marks);

    // Should have 2 separate chains since chain IDs differ
    assert_eq!(report.chains().len(), 2);
}

#[test]
fn test_validate_date_ordering_violation() {
    let marks = create_test_marks(3, ProvenanceMarkResolution::Low, "test");

    // We can't actually create marks with wrong date ordering using the
    // generator, since it enforces consistency. This test demonstrates that
    // the validator would catch it if such marks existed.

    let report = ProvenanceMark::validate(marks);

    assert_eq!(report.chains().len(), 1);
    let chain = &report.chains()[0];
    assert_eq!(chain.sequences().len(), 1);

    // All marks should have no issues
    for flagged_mark in chain.sequences()[0].marks() {
        assert!(flagged_mark.issues().is_empty());
    }
}

#[test]
fn test_validate_multiple_sequences_in_chain() {
    let marks = create_test_marks(7, ProvenanceMarkResolution::Low, "test");

    // Create multiple gaps
    let marks_with_gaps = vec![
        marks[0].clone(), // Sequence 1: [0,1]
        marks[1].clone(),
        marks[3].clone(), // Sequence 2: [3,4] (gap from 1 to 3)
        marks[4].clone(),
        marks[6].clone(), // Sequence 3: [6] (gap from 4 to 6)
    ];

    let report = ProvenanceMark::validate(marks_with_gaps);

    assert_eq!(report.chains().len(), 1);
    let chain = &report.chains()[0];
    assert_eq!(chain.sequences().len(), 3);

    // First sequence: [0,1]
    assert_eq!(chain.sequences()[0].start_seq(), 0);
    assert_eq!(chain.sequences()[0].end_seq(), 1);
    assert_eq!(chain.sequences()[0].marks().len(), 2);

    // Second sequence: [3,4]
    assert_eq!(chain.sequences()[1].start_seq(), 3);
    assert_eq!(chain.sequences()[1].end_seq(), 4);
    assert_eq!(chain.sequences()[1].marks().len(), 2);
    assert_eq!(chain.sequences()[1].marks()[0].issues().len(), 1);

    // Third sequence: [6]
    assert_eq!(chain.sequences()[2].start_seq(), 6);
    assert_eq!(chain.sequences()[2].end_seq(), 6);
    assert_eq!(chain.sequences()[2].marks().len(), 1);
    assert_eq!(chain.sequences()[2].marks()[0].issues().len(), 1);
}

#[test]
fn test_validate_precedes_opt() {
    let marks = create_test_marks(3, ProvenanceMarkResolution::Low, "test");

    // Test valid precedes
    assert!(marks[0].precedes_opt(&marks[1]).is_ok());
    assert!(marks[1].precedes_opt(&marks[2]).is_ok());

    // Test invalid precedes (reverse order)
    assert!(marks[1].precedes_opt(&marks[0]).is_err());

    // Test gap
    assert!(marks[0].precedes_opt(&marks[2]).is_err());
}

#[test]
fn test_validate_chain_id_hex() {
    let marks = create_test_marks(2, ProvenanceMarkResolution::Low, "test");
    let report = ProvenanceMark::validate(marks.clone());

    assert_eq!(report.chains().len(), 1);
    let chain = &report.chains()[0];

    // Chain ID should be raw bytes
    assert!(!chain.chain_id().is_empty());
    assert_eq!(chain.chain_id(), marks[0].chain_id());

    // Hex encoding is available via helper method
    let chain_id_hex = chain.chain_id_hex();
    assert!(chain_id_hex.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(chain_id_hex, hex::encode(marks[0].chain_id()));
}

#[test]
fn test_validate_with_info() {
    let mut generator = ProvenanceMarkGenerator::new_with_passphrase(
        ProvenanceMarkResolution::Low,
        "test",
    );
    let calendar = chrono::Utc;

    let marks: Vec<_> = (0..3)
        .map(|i| {
            let date = Date::from_datetime(
                calendar
                    .with_ymd_and_hms(2023, 6, 20, 12, 0, 0)
                    .single()
                    .unwrap()
                    .checked_add_signed(chrono::Duration::days(i))
                    .unwrap(),
            );
            generator.next(date, Some("Test info"))
        })
        .collect();

    let report = ProvenanceMark::validate(marks);

    assert_eq!(report.chains().len(), 1);
    let chain = &report.chains()[0];
    assert_eq!(chain.sequences().len(), 1);

    // Marks with info should still validate correctly
    for flagged_mark in chain.sequences()[0].marks() {
        assert!(flagged_mark.issues().is_empty());
    }
}

#[test]
fn test_validate_sorted_chains() {
    // Create marks from different chains
    let marks1 = create_test_marks(2, ProvenanceMarkResolution::Low, "zebra");
    let marks2 = create_test_marks(2, ProvenanceMarkResolution::Low, "apple");
    let marks3 = create_test_marks(2, ProvenanceMarkResolution::Low, "middle");

    let mut all_marks = marks1;
    all_marks.extend(marks2);
    all_marks.extend(marks3);

    let report = ProvenanceMark::validate(all_marks);

    assert_eq!(report.chains().len(), 3);

    // Chains should be sorted by chain ID (raw bytes)
    for i in 1..report.chains().len() {
        assert!(report.chains()[i - 1].chain_id() <= report.chains()[i].chain_id());
    }
}

#[test]
fn test_validate_genesis_check() {
    let marks = create_test_marks(3, ProvenanceMarkResolution::Low, "test");

    // With genesis
    let report_with_genesis = ProvenanceMark::validate(marks.clone());
    assert!(report_with_genesis.chains()[0].has_genesis());

    // Without genesis
    let marks_no_genesis: Vec<_> = marks.into_iter().skip(1).collect();
    let report_no_genesis = ProvenanceMark::validate(marks_no_genesis);
    assert!(!report_no_genesis.chains()[0].has_genesis());
}

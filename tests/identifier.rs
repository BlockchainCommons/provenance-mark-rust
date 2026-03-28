use chrono::TimeZone;
use dcbor::Date;
use provenance_mark::*;

fn make_test_marks(count: usize) -> Vec<ProvenanceMark> {
    let generator = ProvenanceMarkGenerator::new_with_passphrase(
        ProvenanceMarkResolution::Low,
        "Wolf",
    );
    let calendar = chrono::Utc;
    let mut encoded = serde_json::to_string(&generator).unwrap();

    (0..count)
        .map(|i| {
            let mut g: ProvenanceMarkGenerator =
                serde_json::from_str(&encoded).unwrap();
            let date = Date::from_datetime(
                calendar
                    .with_ymd_and_hms(2023, 6, 20, 12, 0, 0)
                    .single()
                    .unwrap()
                    .checked_add_signed(chrono::Duration::days(i as i64))
                    .unwrap(),
            );
            let mark = g.next(date, None::<&str>);
            encoded = serde_json::to_string(&g).unwrap();
            mark
        })
        .collect()
}

fn make_marks_for_resolution(
    res: ProvenanceMarkResolution,
    count: usize,
) -> Vec<ProvenanceMark> {
    let generator = ProvenanceMarkGenerator::new_with_passphrase(res, "Wolf");
    let calendar = chrono::Utc;
    let mut encoded = serde_json::to_string(&generator).unwrap();

    (0..count)
        .map(|i| {
            let mut g: ProvenanceMarkGenerator =
                serde_json::from_str(&encoded).unwrap();
            let date = Date::from_datetime(
                calendar
                    .with_ymd_and_hms(2023, 6, 20, 12, 0, 0)
                    .single()
                    .unwrap()
                    .checked_add_signed(chrono::Duration::days(i as i64))
                    .unwrap(),
            );
            let mark = g.next(date, None::<&str>);
            encoded = serde_json::to_string(&g).unwrap();
            mark
        })
        .collect()
}

// --- identifier_hash ---

#[test]
fn test_identifier_hash_returns_32_bytes() {
    for res in [
        ProvenanceMarkResolution::Low,
        ProvenanceMarkResolution::Medium,
        ProvenanceMarkResolution::Quartile,
        ProvenanceMarkResolution::High,
    ] {
        let marks = make_marks_for_resolution(res, 3);
        for mark in &marks {
            let ih = mark.identifier_hash();
            assert_eq!(ih.len(), 32);
        }
    }
}

#[test]
fn test_identifier_hash_preserves_hash_prefix() {
    for res in [
        ProvenanceMarkResolution::Low,
        ProvenanceMarkResolution::Medium,
        ProvenanceMarkResolution::Quartile,
        ProvenanceMarkResolution::High,
    ] {
        let marks = make_marks_for_resolution(res, 3);
        for mark in &marks {
            let ih = mark.identifier_hash();
            let hash = mark.hash();
            assert_eq!(
                &ih[..hash.len()],
                hash,
                "identifier_hash must start with the stored hash for {res}"
            );
        }
    }
}

// --- backward compatibility ---

#[test]
fn test_identifier_n_4_matches_identifier() {
    let marks = make_test_marks(5);
    for mark in &marks {
        assert_eq!(mark.identifier_n(4), mark.identifier());
    }
}

#[test]
fn test_bytewords_identifier_n_4_matches_bytewords_identifier() {
    for res in [
        ProvenanceMarkResolution::Low,
        ProvenanceMarkResolution::Medium,
        ProvenanceMarkResolution::Quartile,
        ProvenanceMarkResolution::High,
    ] {
        let marks = make_marks_for_resolution(res, 5);
        for mark in &marks {
            assert_eq!(
                mark.bytewords_identifier_n(4, false),
                mark.bytewords_identifier(false),
                "backward compat failed for {res}"
            );
            assert_eq!(
                mark.bytewords_identifier_n(4, true),
                mark.bytewords_identifier(true),
            );
        }
    }
}

#[test]
fn test_bytemoji_identifier_n_4_matches_bytemoji_identifier() {
    let marks = make_test_marks(5);
    for mark in &marks {
        assert_eq!(
            mark.bytemoji_identifier_n(4, false),
            mark.bytemoji_identifier(false),
        );
    }
}

// --- parameterized identifiers ---

#[test]
fn test_bytewords_identifier_n_word_count() {
    let marks = make_test_marks(3);
    let mark = &marks[0];

    for n in 4..=32 {
        let id = mark.bytewords_identifier_n(n, false);
        let words: Vec<&str> = id.split(' ').collect();
        assert_eq!(words.len(), n, "expected {n} words, got {}", words.len());
    }
}

#[test]
fn test_bytewords_identifier_n_prefix_extends_shorter() {
    let marks = make_test_marks(1);
    let mark = &marks[0];

    let short = mark.bytewords_identifier_n(4, false);
    let long = mark.bytewords_identifier_n(8, false);
    assert!(
        long.starts_with(&short),
        "8-word identifier must start with 4-word identifier"
    );
}

#[test]
fn test_identifier_n_hex_length() {
    let marks = make_test_marks(1);
    let mark = &marks[0];

    for n in 4..=32 {
        let hex_id = mark.identifier_n(n);
        assert_eq!(hex_id.len(), n * 2, "hex identifier should be {n}*2 chars");
    }
}

// --- panic on invalid input ---

#[test]
#[should_panic(expected = "word_count must be 4..=32")]
fn test_bytewords_identifier_n_panics_below_4() {
    let marks = make_test_marks(1);
    marks[0].bytewords_identifier_n(3, false);
}

#[test]
#[should_panic(expected = "word_count must be 4..=32")]
fn test_bytewords_identifier_n_panics_above_32() {
    let marks = make_test_marks(1);
    marks[0].bytewords_identifier_n(33, false);
}

#[test]
#[should_panic(expected = "byte_count must be 4..=32")]
fn test_identifier_n_panics_below_4() {
    let marks = make_test_marks(1);
    marks[0].identifier_n(0);
}

#[test]
#[should_panic(expected = "word_count must be 4..=32")]
fn test_bytemoji_identifier_n_panics_above_32() {
    let marks = make_test_marks(1);
    marks[0].bytemoji_identifier_n(33, false);
}

// --- disambiguation: no collisions ---

#[test]
fn test_disambiguated_no_collisions() {
    let marks = make_test_marks(5);
    let refs: Vec<&ProvenanceMark> = marks.iter().collect();
    let ids = ProvenanceMark::disambiguated_bytewords_identifiers(&refs, false);

    assert_eq!(ids.len(), 5);
    for id in &ids {
        let words: Vec<&str> = id.split(' ').collect();
        assert_eq!(words.len(), 4, "non-colliding marks should get 4 words");
    }
}

#[test]
fn test_disambiguated_empty() {
    let ids =
        ProvenanceMark::disambiguated_bytewords_identifiers(&[], false);
    assert!(ids.is_empty());
}

#[test]
fn test_disambiguated_single_mark() {
    let marks = make_test_marks(1);
    let refs: Vec<&ProvenanceMark> = marks.iter().collect();
    let ids = ProvenanceMark::disambiguated_bytewords_identifiers(&refs, false);

    assert_eq!(ids.len(), 1);
    let words: Vec<&str> = ids[0].split(' ').collect();
    assert_eq!(words.len(), 4);
}

// --- disambiguation: with collisions ---

#[test]
fn test_disambiguated_selective_extension() {
    // Generate many marks and find a pair that can be made to "collide"
    // by replacing the first mark's hash prefix with the second's.
    // Since we can't easily construct real collisions, we test the
    // disambiguation logic by serializing marks via JSON and tampering
    // with the hash field to force a collision.
    let marks = make_test_marks(5);

    // Verify that non-colliding marks get 4 words
    let refs: Vec<&ProvenanceMark> = marks.iter().collect();
    let ids = ProvenanceMark::disambiguated_bytewords_identifiers(&refs, false);
    for id in &ids {
        let words: Vec<&str> = id.split(' ').collect();
        assert_eq!(words.len(), 4);
    }

    // Now include a duplicate mark (same mark twice) to force collision
    let refs_with_dup: Vec<&ProvenanceMark> =
        vec![&marks[0], &marks[1], &marks[2], &marks[0]];
    let ids =
        ProvenanceMark::disambiguated_bytewords_identifiers(&refs_with_dup, false);

    assert_eq!(ids.len(), 4);

    // marks[1] and marks[2] should still have 4 words (no collision)
    let words1: Vec<&str> = ids[1].split(' ').collect();
    let words2: Vec<&str> = ids[2].split(' ').collect();
    assert_eq!(words1.len(), 4, "non-colliding mark should stay at 4 words");
    assert_eq!(words2.len(), 4, "non-colliding mark should stay at 4 words");

    // The duplicate pair (indices 0 and 3) have identical identifier_hash,
    // so they'll be extended to 32 words (can't disambiguate identical marks)
    let words0: Vec<&str> = ids[0].split(' ').collect();
    let words3: Vec<&str> = ids[3].split(' ').collect();
    assert_eq!(words0.len(), 32, "identical marks extend to max");
    assert_eq!(words3.len(), 32, "identical marks extend to max");
    assert_eq!(ids[0], ids[3], "identical marks produce identical identifiers");
}

#[test]
fn test_disambiguated_all_results_unique_except_identical() {
    let marks = make_test_marks(10);
    let refs: Vec<&ProvenanceMark> = marks.iter().collect();
    let ids = ProvenanceMark::disambiguated_bytewords_identifiers(&refs, false);

    let unique: std::collections::HashSet<&str> =
        ids.iter().map(String::as_str).collect();
    assert_eq!(unique.len(), ids.len(), "all identifiers should be unique");
}

// --- disambiguation: bytemoji parity ---

#[test]
fn test_disambiguated_bytemoji_same_prefix_lengths() {
    // Duplicate marks force collision — verify both encodings extend equally
    let marks = make_test_marks(3);
    let refs: Vec<&ProvenanceMark> =
        vec![&marks[0], &marks[1], &marks[0]];

    let word_ids =
        ProvenanceMark::disambiguated_bytewords_identifiers(&refs, false);
    let emoji_ids =
        ProvenanceMark::disambiguated_bytemoji_identifiers(&refs, false);

    assert_eq!(word_ids.len(), emoji_ids.len());

    for (w, e) in word_ids.iter().zip(emoji_ids.iter()) {
        let word_count = w.split(' ').count();
        let emoji_count = e.split(' ').count();
        assert_eq!(
            word_count, emoji_count,
            "bytewords and bytemoji should use same prefix lengths"
        );
    }
}

// --- prefix flag ---

#[test]
fn test_disambiguated_with_prefix() {
    let marks = make_test_marks(3);
    let refs: Vec<&ProvenanceMark> = marks.iter().collect();

    let ids_no_prefix =
        ProvenanceMark::disambiguated_bytewords_identifiers(&refs, false);
    let ids_prefix =
        ProvenanceMark::disambiguated_bytewords_identifiers(&refs, true);

    for (no_pfx, pfx) in ids_no_prefix.iter().zip(ids_prefix.iter()) {
        assert!(pfx.starts_with("🅟 "));
        assert_eq!(&pfx[5..], no_pfx.as_str());
    }
}

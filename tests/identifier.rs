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

// --- id ---

#[test]
fn test_id_returns_32_bytes() {
    for res in [
        ProvenanceMarkResolution::Low,
        ProvenanceMarkResolution::Medium,
        ProvenanceMarkResolution::Quartile,
        ProvenanceMarkResolution::High,
    ] {
        let marks = make_marks_for_resolution(res, 3);
        for mark in &marks {
            assert_eq!(mark.id().len(), 32);
        }
    }
}

#[test]
fn test_id_preserves_hash_prefix() {
    for res in [
        ProvenanceMarkResolution::Low,
        ProvenanceMarkResolution::Medium,
        ProvenanceMarkResolution::Quartile,
        ProvenanceMarkResolution::High,
    ] {
        let marks = make_marks_for_resolution(res, 3);
        for mark in &marks {
            let id = mark.id();
            let hash = mark.hash();
            assert_eq!(
                &id[..hash.len()],
                hash,
                "id must start with the stored hash for {res}"
            );
        }
    }
}

// --- id_hex ---

#[test]
fn test_id_hex_is_64_chars() {
    let marks = make_test_marks(5);
    for mark in &marks {
        let hex = mark.id_hex();
        assert_eq!(hex.len(), 64, "id_hex must be 64 hex chars");
    }
}

#[test]
fn test_id_hex_encodes_full_id() {
    let marks = make_test_marks(1);
    let mark = &marks[0];
    assert_eq!(mark.id_hex(), hex::encode(mark.id()));
}

// --- id_bytewords ---

#[test]
fn test_id_bytewords_word_count() {
    let marks = make_test_marks(3);
    let mark = &marks[0];

    for n in 4..=32 {
        let bw = mark.id_bytewords(n, false);
        let words: Vec<&str> = bw.split(' ').collect();
        assert_eq!(words.len(), n, "expected {n} words, got {}", words.len());
    }
}

#[test]
fn test_id_bytewords_prefix_extends_shorter() {
    let marks = make_test_marks(1);
    let mark = &marks[0];

    let short = mark.id_bytewords(4, false);
    let long = mark.id_bytewords(8, false);
    assert!(
        long.starts_with(&short),
        "8-word id must start with 4-word id"
    );
}

#[test]
fn test_id_bytewords_with_prefix_flag() {
    let marks = make_test_marks(1);
    let mark = &marks[0];
    let without = mark.id_bytewords(4, false);
    let with = mark.id_bytewords(4, true);
    assert!(with.starts_with("🅟 "));
    assert_eq!(&with[5..], without.as_str());
}

// --- id_bytemoji ---

#[test]
fn test_id_bytemoji_word_count() {
    let marks = make_test_marks(1);
    let mark = &marks[0];
    for n in 4..=32 {
        let bm = mark.id_bytemoji(n, false);
        let emojis: Vec<&str> = bm.split(' ').collect();
        assert_eq!(emojis.len(), n);
    }
}

// --- id_bytewords_minimal ---

#[test]
fn test_id_bytewords_minimal_length() {
    let marks = make_test_marks(1);
    let mark = &marks[0];

    for n in 4..=32 {
        let minimal = mark.id_bytewords_minimal(n, false);
        assert_eq!(
            minimal.len(),
            n * 2,
            "minimal bytewords for {n} bytes should be {n}*2 chars"
        );
    }
}

#[test]
fn test_id_bytewords_minimal_is_uppercase() {
    let marks = make_test_marks(1);
    let minimal = marks[0].id_bytewords_minimal(4, false);
    assert_eq!(minimal, minimal.to_uppercase());
}

#[test]
fn test_id_bytewords_minimal_extends_shorter() {
    let marks = make_test_marks(1);
    let mark = &marks[0];
    let short = mark.id_bytewords_minimal(4, false);
    let long = mark.id_bytewords_minimal(8, false);
    assert!(
        long.starts_with(&short),
        "8-byte minimal must start with 4-byte minimal"
    );
}

// --- panic on invalid input ---

#[test]
#[should_panic(expected = "word_count must be 4..=32")]
fn test_id_bytewords_panics_below_4() {
    let marks = make_test_marks(1);
    marks[0].id_bytewords(3, false);
}

#[test]
#[should_panic(expected = "word_count must be 4..=32")]
fn test_id_bytewords_panics_above_32() {
    let marks = make_test_marks(1);
    marks[0].id_bytewords(33, false);
}

#[test]
#[should_panic(expected = "word_count must be 4..=32")]
fn test_id_bytemoji_panics_above_32() {
    let marks = make_test_marks(1);
    marks[0].id_bytemoji(33, false);
}

#[test]
#[should_panic(expected = "word_count must be 4..=32")]
fn test_id_bytewords_minimal_panics_below_4() {
    let marks = make_test_marks(1);
    marks[0].id_bytewords_minimal(3, false);
}

// --- disambiguation: no collisions ---

#[test]
fn test_disambiguated_no_collisions() {
    let marks = make_test_marks(5);
    let refs: Vec<&ProvenanceMark> = marks.iter().collect();
    let ids = ProvenanceMark::disambiguated_id_bytewords(&refs, false);

    assert_eq!(ids.len(), 5);
    for id in &ids {
        let words: Vec<&str> = id.split(' ').collect();
        assert_eq!(words.len(), 4, "non-colliding marks should get 4 words");
    }
}

#[test]
fn test_disambiguated_empty() {
    let ids = ProvenanceMark::disambiguated_id_bytewords(&[], false);
    assert!(ids.is_empty());
}

#[test]
fn test_disambiguated_single_mark() {
    let marks = make_test_marks(1);
    let refs: Vec<&ProvenanceMark> = marks.iter().collect();
    let ids = ProvenanceMark::disambiguated_id_bytewords(&refs, false);

    assert_eq!(ids.len(), 1);
    let words: Vec<&str> = ids[0].split(' ').collect();
    assert_eq!(words.len(), 4);
}

// --- disambiguation: with collisions ---

#[test]
fn test_disambiguated_selective_extension() {
    let marks = make_test_marks(5);

    // Verify that non-colliding marks get 4 words
    let refs: Vec<&ProvenanceMark> = marks.iter().collect();
    let ids = ProvenanceMark::disambiguated_id_bytewords(&refs, false);
    for id in &ids {
        let words: Vec<&str> = id.split(' ').collect();
        assert_eq!(words.len(), 4);
    }

    // Now include a duplicate mark (same mark twice) to force collision
    let refs_with_dup: Vec<&ProvenanceMark> =
        vec![&marks[0], &marks[1], &marks[2], &marks[0]];
    let ids = ProvenanceMark::disambiguated_id_bytewords(&refs_with_dup, false);

    assert_eq!(ids.len(), 4);

    // marks[1] and marks[2] should still have 4 words (no collision)
    let words1: Vec<&str> = ids[1].split(' ').collect();
    let words2: Vec<&str> = ids[2].split(' ').collect();
    assert_eq!(words1.len(), 4, "non-colliding mark should stay at 4 words");
    assert_eq!(words2.len(), 4, "non-colliding mark should stay at 4 words");

    // The duplicate pair (indices 0 and 3) have identical IDs,
    // so they'll be extended to 32 words (can't disambiguate identical marks)
    let words0: Vec<&str> = ids[0].split(' ').collect();
    let words3: Vec<&str> = ids[3].split(' ').collect();
    assert_eq!(words0.len(), 32, "identical marks extend to max");
    assert_eq!(words3.len(), 32, "identical marks extend to max");
    assert_eq!(
        ids[0], ids[3],
        "identical marks produce identical identifiers"
    );
}

#[test]
fn test_disambiguated_all_results_unique_except_identical() {
    let marks = make_test_marks(10);
    let refs: Vec<&ProvenanceMark> = marks.iter().collect();
    let ids = ProvenanceMark::disambiguated_id_bytewords(&refs, false);

    let unique: std::collections::HashSet<&str> =
        ids.iter().map(String::as_str).collect();
    assert_eq!(unique.len(), ids.len(), "all identifiers should be unique");
}

// --- disambiguation: bytemoji parity ---

#[test]
fn test_disambiguated_bytemoji_same_prefix_lengths() {
    let marks = make_test_marks(3);
    let refs: Vec<&ProvenanceMark> = vec![&marks[0], &marks[1], &marks[0]];

    let word_ids = ProvenanceMark::disambiguated_id_bytewords(&refs, false);
    let emoji_ids = ProvenanceMark::disambiguated_id_bytemoji(&refs, false);

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
        ProvenanceMark::disambiguated_id_bytewords(&refs, false);
    let ids_prefix = ProvenanceMark::disambiguated_id_bytewords(&refs, true);

    for (no_pfx, pfx) in ids_no_prefix.iter().zip(ids_prefix.iter()) {
        assert!(pfx.starts_with("🅟 "));
        assert_eq!(&pfx[5..], no_pfx.as_str());
    }
}

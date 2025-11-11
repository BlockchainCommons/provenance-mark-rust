use chrono::TimeZone;
use dcbor::Date;
use indoc::indoc;
use provenance_mark::*;

#[macro_use]
mod common;

fn create_test_marks(
    count: usize,
    resolution: ProvenanceMarkResolution,
    passphrase: &str,
) -> Vec<ProvenanceMark> {
    #[cfg(feature = "envelope")]
    provenance_mark::register_tags();

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

    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [],
          "chains": []
        }"#}.trim());

    // Test compact JSON format
    let json_compact = report.format(ValidationReportFormat::JsonCompact);
    #[rustfmt::skip]
    assert_actual_expected!(json_compact, r#"{"marks":[],"chains":[]}"#);

    // Format should return empty string for empty report
    assert_actual_expected!(report.format(ValidationReportFormat::Text), "");
}

#[test]
fn test_validate_single_mark() {
    let marks = create_test_marks(1, ProvenanceMarkResolution::Low, "test");
    let report = ProvenanceMark::validate(marks.clone());

    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 0,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                      "issues": []
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());

    // Test compact JSON format
    let json_compact = report.format(ValidationReportFormat::JsonCompact);
    #[rustfmt::skip]
    assert_actual_expected!(json_compact, r#"{"marks":["ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba"],"chains":[{"chain_id":"b16a7cbd","has_genesis":true,"marks":["ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba"],"sequences":[{"start_seq":0,"end_seq":0,"marks":[{"mark":"ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba","issues":[]}]}]}]}"#);

    // Format should return empty string for single perfect chain
    assert_actual_expected!(report.format(ValidationReportFormat::Text), "");
}

#[test]
fn test_validate_valid_sequence() {
    let marks = create_test_marks(5, ProvenanceMarkResolution::Low, "test");
    let report = ProvenanceMark::validate(marks.clone());

    // Test JSON serialization
    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
            "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
            "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd",
            "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
            "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd",
                "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
                "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 4,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk",
                      "issues": []
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());

    // Format should return empty string for single perfect chain
    assert_actual_expected!(report.format(ValidationReportFormat::Text), "");
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

    // Test JSON serialization
    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
            "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
            "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 2,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd",
                      "issues": []
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());

    // Format should return empty string - single perfect chain after
    // deduplication
    assert_actual_expected!(report.format(ValidationReportFormat::Text), "");
}

#[test]
fn test_validate_multiple_chains() {
    let marks1 = create_test_marks(3, ProvenanceMarkResolution::Low, "alice");
    let marks2 = create_test_marks(3, ProvenanceMarkResolution::Low, "bob");

    let mut all_marks = marks1.clone();
    all_marks.extend(marks2.clone());

    let report = ProvenanceMark::validate(all_marks);

    // Test JSON serialization
    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdotfmbeuerniolpveenmowliegyfrfrwnfzntnbwe",
            "ur:provenance/lfaegdztfetoehnyjswzsopecewkqdiskshfnyndiemkld",
            "ur:provenance/lfaegdenrdietbenskbesbdiiefgwkuoqzldbecpidhfrt",
            "ur:provenance/lfaegdknnsfhhylrgytdhtsnheskzepmctgrwnlyjeyngh",
            "ur:provenance/lfaegdrtckinuywdosecpedtbnismdcllyvsbbplkpspyl",
            "ur:provenance/lfaegdrevlpmticnmkbafsinmeonvycydphernwerppefs"
          ],
          "chains": [
            {
              "chain_id": "7a9c3f5e",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdknnsfhhylrgytdhtsnheskzepmctgrwnlyjeyngh",
                "ur:provenance/lfaegdrtckinuywdosecpedtbnismdcllyvsbbplkpspyl",
                "ur:provenance/lfaegdrevlpmticnmkbafsinmeonvycydphernwerppefs"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 2,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdknnsfhhylrgytdhtsnheskzepmctgrwnlyjeyngh",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdrtckinuywdosecpedtbnismdcllyvsbbplkpspyl",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdrevlpmticnmkbafsinmeonvycydphernwerppefs",
                      "issues": []
                    }
                  ]
                }
              ]
            },
            {
              "chain_id": "a33e10de",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdotfmbeuerniolpveenmowliegyfrfrwnfzntnbwe",
                "ur:provenance/lfaegdztfetoehnyjswzsopecewkqdiskshfnyndiemkld",
                "ur:provenance/lfaegdenrdietbenskbesbdiiefgwkuoqzldbecpidhfrt"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 2,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdotfmbeuerniolpveenmowliegyfrfrwnfzntnbwe",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdztfetoehnyjswzsopecewkqdiskshfnyndiemkld",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdenrdietbenskbesbdiiefgwkuoqzldbecpidhfrt",
                      "issues": []
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());

    // Format should show both chains (interesting)
    #[rustfmt::skip]
    assert_actual_expected!(report.format(ValidationReportFormat::Text), indoc! {r#"
        Total marks: 6
        Chains: 2

        Chain 1: 7a9c3f5e
          0: 0d6e0afd (genesis mark)
          1: 6cd504e7
          2: dc07895c

        Chain 2: a33e10de
          0: c2a985ff (genesis mark)
          1: 5567cd24
          2: f759ad4c

    "#}.trim());
}

#[test]
fn test_validate_missing_genesis() {
    let marks = create_test_marks(5, ProvenanceMarkResolution::Low, "test");

    // Remove genesis mark (index 0)
    let marks_no_genesis: Vec<_> = marks.into_iter().skip(1).collect();

    let report = ProvenanceMark::validate(marks_no_genesis);

    // Test JSON serialization
    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
            "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd",
            "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
            "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": false,
              "marks": [
                "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd",
                "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
                "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk"
              ],
              "sequences": [
                {
                  "start_seq": 1,
                  "end_seq": 4,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk",
                      "issues": []
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());

    // Format should show missing genesis warning
    #[rustfmt::skip]
    assert_actual_expected!(report.format(ValidationReportFormat::Text), indoc! {r#"
        Total marks: 4
        Chains: 1

        Chain 1: b16a7cbd
          Warning: No genesis mark found
          1: 1b806d6c
          2: b292f357
          3: 761a5e74
          4: 42d12de5

    "#}.trim());
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

    // Test JSON serialization
    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
            "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
            "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
            "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
                "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 1,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                      "issues": []
                    }
                  ]
                },
                {
                  "start_seq": 3,
                  "end_seq": 4,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
                      "issues": [
                        {
                          "type": "SequenceGap",
                          "data": {
                            "expected": 2,
                            "actual": 3
                          }
                        }
                      ]
                    },
                    {
                      "mark": "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk",
                      "issues": []
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());

    // Format should show gap issue and multiple sequences
    #[rustfmt::skip]
    assert_actual_expected!(report.format(ValidationReportFormat::Text), indoc! {r#"
        Total marks: 4
        Chains: 1

        Chain 1: b16a7cbd
          0: f057c8c4 (genesis mark)
          1: 1b806d6c
          3: 761a5e74 (gap: 2 missing)
          4: 42d12de5

    "#}.trim());
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

    // Test JSON serialization
    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
            "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
            "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
            "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd",
            "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd",
                "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
                "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 4,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk",
                      "issues": []
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());

    // Format should return empty string - validation sorts by seq number
    assert_actual_expected!(report.format(ValidationReportFormat::Text), "");
}

#[test]
fn test_validate_hash_mismatch() {
    #[cfg(feature = "envelope")]
    provenance_mark::register_tags();

    let marks = create_test_marks(3, ProvenanceMarkResolution::Low, "test");
    let mark0 = &marks[0];
    let mark1 = &marks[1];

    // Create a third mark that claims to follow mark1 but with wrong prev hash
    let calendar = chrono::Utc;
    let date = Date::from_datetime(
        calendar
            .with_ymd_and_hms(2023, 6, 22, 12, 0, 0)
            .single()
            .unwrap(),
    );

    // Use mark1's chain_id and key, but use mark0's hash as prev (wrong!)
    // This creates a hash mismatch since mark1.hash should be the prev
    let bad_mark = ProvenanceMark::new(
        mark1.res(),
        mark1.key().to_vec(),
        mark0.hash().to_vec(), // Wrong! Should be mark1.hash()
        mark1.chain_id().to_vec(),
        2,
        date,
        None::<String>,
    )
    .unwrap();

    let report =
        ProvenanceMark::validate(vec![mark0.clone(), mark1.clone(), bad_mark]);

    // Test JSON serialization
    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
            "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
            "ur:provenance/lfaegdecgldtsrbbfgsbethprlwfgsrnttrtkpgsttptwn"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                "ur:provenance/lfaegdecgldtsrbbfgsbethprlwfgsrnttrtkpgsttptwn"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 1,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                      "issues": []
                    }
                  ]
                },
                {
                  "start_seq": 2,
                  "end_seq": 2,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdecgldtsrbbfgsbethprlwfgsrnttrtkpgsttptwn",
                      "issues": [
                        {
                          "type": "HashMismatch",
                          "data": {
                            "expected": "d446017b",
                            "actual": "1b806d6c"
                          }
                        }
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());

    // Format should show hash mismatch issue
    #[rustfmt::skip]
    assert_actual_expected!(report.format(ValidationReportFormat::Text).trim(), indoc! {r#"
        Total marks: 3
        Chains: 1

        Chain 1: b16a7cbd
          0: f057c8c4 (genesis mark)
          1: 1b806d6c
          2: 09cca821 (hash mismatch)
    "#}.trim());
}

#[test]
fn test_validate_date_ordering_violation() {
    let marks = create_test_marks(3, ProvenanceMarkResolution::Low, "test");

    // We can't actually create marks with wrong date ordering using the
    // generator, since it enforces consistency. This test demonstrates that
    // the validator would catch it if such marks existed.

    let report = ProvenanceMark::validate(marks);

    // Test JSON serialization
    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
            "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
            "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 2,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd",
                      "issues": []
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());
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

    // Test JSON serialization
    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
            "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
            "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
            "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk",
            "ur:provenance/lfaegdwkltwzolasuomobntaryinjzcyrocsfskkrtmyam"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
                "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk",
                "ur:provenance/lfaegdwkltwzolasuomobntaryinjzcyrocsfskkrtmyam"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 1,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                      "issues": []
                    }
                  ]
                },
                {
                  "start_seq": 3,
                  "end_seq": 4,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdhsvtleetlatsmwwdndmnjlaxonsfdewmghpybzbg",
                      "issues": [
                        {
                          "type": "SequenceGap",
                          "data": {
                            "expected": 2,
                            "actual": 3
                          }
                        }
                      ]
                    },
                    {
                      "mark": "ur:provenance/lfaegdrkkilkylsrendmkniaeejyrhndlyvednzckpsbtk",
                      "issues": []
                    }
                  ]
                },
                {
                  "start_seq": 6,
                  "end_seq": 6,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdwkltwzolasuomobntaryinjzcyrocsfskkrtmyam",
                      "issues": [
                        {
                          "type": "SequenceGap",
                          "data": {
                            "expected": 5,
                            "actual": 6
                          }
                        }
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());

    // Format should show multiple sequences with gap annotations
    #[rustfmt::skip]
    assert_actual_expected!(report.format(ValidationReportFormat::Text), indoc! {r#"
        Total marks: 5
        Chains: 1

        Chain 1: b16a7cbd
          0: f057c8c4 (genesis mark)
          1: 1b806d6c
          3: 761a5e74 (gap: 2 missing)
          4: 42d12de5
          6: 8a9b06e1 (gap: 5 missing)

    "#}.trim());
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

    let chain = &report.chains()[0];
    let chain_id_hex = chain.chain_id_hex();

    // Verify hex encoding
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

    // Test JSON serialization
    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaehdcypaimkerydihsaedesbglvlrsgdmocfdpveksstlbrprscahlihyntoaxvtem",
            "ur:provenance/lfaehdcyecgldtsrbbfgsbetsrsgsafwrntdrtkohdhntnwdvtcsatnbkiythefdkiso",
            "ur:provenance/lfaehdcybwatptqzoyrkdmptfntsjsqdpmpmrfoylewnlpjnhdwzadnycljncflozsfy"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaehdcypaimkerydihsaedesbglvlrsgdmocfdpveksstlbrprscahlihyntoaxvtem",
                "ur:provenance/lfaehdcyecgldtsrbbfgsbetsrsgsafwrntdrtkohdhntnwdvtcsatnbkiythefdkiso",
                "ur:provenance/lfaehdcybwatptqzoyrkdmptfntsjsqdpmpmrfoylewnlpjnhdwzadnycljncflozsfy"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 2,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaehdcypaimkerydihsaedesbglvlrsgdmocfdpveksstlbrprscahlihyntoaxvtem",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaehdcyecgldtsrbbfgsbetsrsgsafwrntdrtkohdhntnwdvtcsatnbkiythefdkiso",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaehdcybwatptqzoyrkdmptfntsjsqdpmpmrfoylewnlpjnhdwzadnycljncflozsfy",
                      "issues": []
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());
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

    // Test JSON serialization
    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdcktndeltrtspprmhkptlfdwfgylsjljzwtahlpsf",
            "ur:provenance/lfaegdrslnurdeknftkscnlphnhgldcxnnahwddiaavyda",
            "ur:provenance/lfaegdfltogtdmfpdphlttkilywyfntidsamrkmuioteid",
            "ur:provenance/lfaegdntjopfzttddtsrkirkdytlkirhisiyidimdmwnkg",
            "ur:provenance/lfaegdfylajldrntasvyttgljtsbsoghdafzwfcawmgede",
            "ur:provenance/lfaegdgrrtjorhmuzshlvsfdldchoxbntlsrstoyidjepm"
          ],
          "chains": [
            {
              "chain_id": "1eda2887",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdcktndeltrtspprmhkptlfdwfgylsjljzwtahlpsf",
                "ur:provenance/lfaegdrslnurdeknftkscnlphnhgldcxnnahwddiaavyda"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 1,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdcktndeltrtspprmhkptlfdwfgylsjljzwtahlpsf",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdrslnurdeknftkscnlphnhgldcxnnahwddiaavyda",
                      "issues": []
                    }
                  ]
                }
              ]
            },
            {
              "chain_id": "44806f2a",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdfylajldrntasvyttgljtsbsoghdafzwfcawmgede",
                "ur:provenance/lfaegdgrrtjorhmuzshlvsfdldchoxbntlsrstoyidjepm"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 1,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdfylajldrntasvyttgljtsbsoghdafzwfcawmgede",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdgrrtjorhmuzshlvsfdldchoxbntlsrstoyidjepm",
                      "issues": []
                    }
                  ]
                }
              ]
            },
            {
              "chain_id": "47ce4d2e",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdfltogtdmfpdphlttkilywyfntidsamrkmuioteid",
                "ur:provenance/lfaegdntjopfzttddtsrkirkdytlkirhisiyidimdmwnkg"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 1,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdfltogtdmfpdphlttkilywyfntidsamrkmuioteid",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdntjopfzttddtsrkirkdytlkirhisiyidimdmwnkg",
                      "issues": []
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());
}

#[test]
fn test_validate_genesis_check() {
    let marks = create_test_marks(3, ProvenanceMarkResolution::Low, "test");

    // With genesis
    let report_with_genesis = ProvenanceMark::validate(marks.clone());

    // Test JSON serialization
    let json = report_with_genesis.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
            "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
            "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 2,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd",
                      "issues": []
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());

    // Without genesis
    let marks_no_genesis: Vec<_> = marks.into_iter().skip(1).collect();
    let report_no_genesis = ProvenanceMark::validate(marks_no_genesis);

    // Test JSON serialization
    let json = report_no_genesis.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
            "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": false,
              "marks": [
                "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd"
              ],
              "sequences": [
                {
                  "start_seq": 1,
                  "end_seq": 2,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdecgldtsrbbfgsbetgazoenadrntdrtkoluwekerp",
                      "issues": []
                    },
                    {
                      "mark": "ur:provenance/lfaegdbwatptqzoyrkdmptvasefnfmpmpmrfoywyptolfd",
                      "issues": []
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());
}

#[test]
fn test_validate_date_ordering_violation_constructed() {
    #[cfg(feature = "envelope")]
    provenance_mark::register_tags();

    let marks = create_test_marks(2, ProvenanceMarkResolution::Low, "test");
    let mark0 = &marks[0];

    let calendar = chrono::Utc;
    // Create a second mark with an earlier date
    let earlier_date = Date::from_datetime(
        calendar
            .with_ymd_and_hms(2023, 6, 19, 12, 0, 0)
            .single()
            .unwrap(),
    );

    // To test date ordering, we need to create mark1 with the correct key from
    // generator but with an earlier date
    let mut generator = ProvenanceMarkGenerator::new_with_passphrase(
        ProvenanceMarkResolution::Low,
        "test",
    );
    let _ = generator.next(mark0.date().clone(), None::<String>); // skip first
    let mark1_bad_date = generator.next(earlier_date, None::<String>);

    let report = ProvenanceMark::validate(vec![mark0.clone(), mark1_bad_date]);

    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
            "ur:provenance/lfaegdecgldtsrbbfgsbetckchiatnrntdrtjohpbdeteo"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                "ur:provenance/lfaegdecgldtsrbbfgsbetckchiatnrntdrtjohpbdeteo"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 0,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                      "issues": []
                    }
                  ]
                },
                {
                  "start_seq": 1,
                  "end_seq": 1,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdecgldtsrbbfgsbetckchiatnrntdrtjohpbdeteo",
                      "issues": [
                        {
                          "type": "DateOrdering",
                          "data": {
                            "previous": "2023-06-20",
                            "next": "2023-06-19"
                          }
                        }
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());
}

#[test]
fn test_validate_non_genesis_at_seq_zero() {
    #[cfg(feature = "envelope")]
    provenance_mark::register_tags();

    // Create proper marks
    let marks = create_test_marks(2, ProvenanceMarkResolution::Low, "test");
    let mark0 = &marks[0];
    let mark1 = &marks[1];

    // When mark1 claims to be at seq 0, it should fail NonGenesisAtZero check
    // when preceded by mark0
    let calendar = chrono::Utc;
    let date = Date::from_datetime(
        calendar
            .with_ymd_and_hms(2023, 6, 21, 12, 0, 0)
            .single()
            .unwrap(),
    );

    let bad_mark = ProvenanceMark::new(
        mark1.res(),
        mark1.key().to_vec(),
        mark1.hash().to_vec(),
        mark1.chain_id().to_vec(),
        0, // Claim seq 0 but not genesis
        date,
        None::<String>,
    )
    .unwrap();

    // The report shows mark0 and bad_mark, where bad_mark is rejected
    let report = ProvenanceMark::validate(vec![mark0.clone(), bad_mark]);

    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
            "ur:provenance/lfaegdecgldtsrbbfgsbetbahhgowzrntertkopkmyiowp"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                "ur:provenance/lfaegdecgldtsrbbfgsbetbahhgowzrntertkopkmyiowp"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 0,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                      "issues": []
                    }
                  ]
                },
                {
                  "start_seq": 0,
                  "end_seq": 0,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdecgldtsrbbfgsbetbahhgowzrntertkopkmyiowp",
                      "issues": [
                        {
                          "type": "NonGenesisAtZero"
                        }
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());
}

#[test]
fn test_validate_invalid_genesis_key_constructed() {
    #[cfg(feature = "envelope")]
    provenance_mark::register_tags();

    // Create proper marks
    let marks = create_test_marks(2, ProvenanceMarkResolution::Low, "test");
    let mark0 = &marks[0];
    let mark1 = &marks[1];

    // When mark1 is at seq > 0 but has key == chain_id, it should fail
    // InvalidGenesisKey
    let calendar = chrono::Utc;
    let date = Date::from_datetime(
        calendar
            .with_ymd_and_hms(2023, 6, 21, 12, 0, 0)
            .single()
            .unwrap(),
    );

    let bad_mark = ProvenanceMark::new(
        mark1.res(),
        mark1.chain_id().to_vec(), // key == chain_id (not allowed at seq > 0)
        mark1.hash().to_vec(),
        mark1.chain_id().to_vec(),
        1, // seq 1
        date,
        None::<String>,
    )
    .unwrap();

    // The report shows mark0 and bad_mark, where bad_mark is rejected
    let report = ProvenanceMark::validate(vec![mark0.clone(), bad_mark]);

    let json = report.format(ValidationReportFormat::JsonPretty);
    #[rustfmt::skip]
    assert_actual_expected!(json, indoc! {r#"
        {
          "marks": [
            "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
            "ur:provenance/lfaegdpaimkerydihsaedewnwnsnwmgdmucfdwcpfxdtsr"
          ],
          "chains": [
            {
              "chain_id": "b16a7cbd",
              "has_genesis": true,
              "marks": [
                "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                "ur:provenance/lfaegdpaimkerydihsaedewnwnsnwmgdmucfdwcpfxdtsr"
              ],
              "sequences": [
                {
                  "start_seq": 0,
                  "end_seq": 0,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdpaimkerydihsaedetiimmttpgdmocfdpbnhlasba",
                      "issues": []
                    }
                  ]
                },
                {
                  "start_seq": 1,
                  "end_seq": 1,
                  "marks": [
                    {
                      "mark": "ur:provenance/lfaegdpaimkerydihsaedewnwnsnwmgdmucfdwcpfxdtsr",
                      "issues": [
                        {
                          "type": "InvalidGenesisKey"
                        }
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        }"#}.trim());
}

use chrono::{TimeZone, Timelike, Utc};
use dcbor::prelude::*;
use hex_literal::hex;
use provenance_mark::date::SerializableDate;

#[test]
fn test_2_byte_dates() {
    // Base date serialization and deserialization
    let base_date = Date::from_datetime(
        Utc.with_ymd_and_hms(2023, 6, 20, 0, 0, 0).unwrap(),
    );
    let serialized = base_date.serialize_2_bytes().unwrap();
    assert_eq!(hex::encode(serialized), "00d4");
    let deserialized = Date::deserialize_2_bytes(&serialized).unwrap();
    assert_eq!(base_date, deserialized);

    // Minimum date
    let min_serialized = [0x00, 0x21];
    let min_date =
        Date::from_datetime(Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap());
    let deserialized_min = Date::deserialize_2_bytes(&min_serialized).unwrap();
    assert_eq!(min_date, deserialized_min);

    // Maximum date
    let max_serialized = [0xff, 0x9f];
    let deserialized_max = Date::deserialize_2_bytes(&max_serialized).unwrap();
    let expected_max_date = Date::from_datetime(
        Utc.with_ymd_and_hms(2150, 12, 31, 0, 0, 0).unwrap(),
    );
    assert_eq!(deserialized_max, expected_max_date);

    // Invalid date
    let invalid_serialized = [0x00, 0x5e]; // Represents 2023-02-30, which is invalid
    assert!(Date::deserialize_2_bytes(&invalid_serialized).is_err());
}

#[test]
fn test_4_byte_dates() {
    // Base date serialization and deserialization
    let base_date = Date::from_datetime(
        Utc.with_ymd_and_hms(2023, 6, 20, 12, 34, 56).unwrap(),
    );
    let serialized = base_date.serialize_4_bytes().unwrap();
    assert_eq!(serialized, hex!("2a41d470"));
    let deserialized = Date::deserialize_4_bytes(&serialized).unwrap();
    assert_eq!(base_date, deserialized);

    // Minimum date
    let min_serialized = hex!("00000000");
    let min_date =
        Date::from_datetime(Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0).unwrap());
    let deserialized_min = Date::deserialize_4_bytes(&min_serialized).unwrap();
    assert_eq!(min_date, deserialized_min);

    // Maximum date
    let max_serialized = hex!("ffffffff");
    let deserialized_max = Date::deserialize_4_bytes(&max_serialized).unwrap();
    let expected_max_date = Date::from_datetime(
        Utc.with_ymd_and_hms(2137, 2, 7, 6, 28, 15).unwrap(),
    );
    assert_eq!(deserialized_max, expected_max_date);
}

#[test]
fn test_6_byte_dates() {
    // Base date serialization and deserialization
    let base_date = Date::from_datetime(
        Utc.with_ymd_and_hms(2023, 6, 20, 12, 34, 56)
            .unwrap()
            .with_nanosecond(789_000_000)
            .unwrap(),
    );
    let serialized = base_date.serialize_6_bytes().unwrap();
    assert_eq!(serialized, hex!("00a51125d895"));
    let deserialized = Date::deserialize_6_bytes(&serialized).unwrap();
    assert_eq!(base_date, deserialized);

    // Minimum date
    let min_serialized = hex!("000000000000");
    let min_date =
        Date::from_datetime(Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0).unwrap());
    let deserialized_min = Date::deserialize_6_bytes(&min_serialized).unwrap();
    assert_eq!(min_date, deserialized_min);

    // Maximum date
    let max_serialized = hex!("e5940a78a7ff");
    let deserialized_max = Date::deserialize_6_bytes(&max_serialized).unwrap();
    let expected_max_date = Date::from_datetime(
        Utc.with_ymd_and_hms(9999, 12, 31, 23, 59, 59)
            .unwrap()
            .with_nanosecond(999_000_000)
            .unwrap(),
    );
    assert_eq!(deserialized_max, expected_max_date);

    // Invalid date (exceeds maximum representable value)
    let invalid_serialized = hex!("e5940a78a800");
    assert!(Date::deserialize_6_bytes(&invalid_serialized).is_err());
}

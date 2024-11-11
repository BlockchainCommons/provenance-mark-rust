use chrono::{ Datelike, Duration, TimeZone, Utc };
use dcbor::Date;
use anyhow::{ Result, bail };

pub trait SerializableDate: Sized {
    fn serialize_2_bytes(&self) -> Result<[u8; 2]>;
    fn deserialize_2_bytes(bytes: &[u8; 2]) -> Result<Self>;

    fn serialize_4_bytes(&self) -> Result<[u8; 4]>;
    fn deserialize_4_bytes(bytes: &[u8; 4]) -> Result<Self>;

    fn serialize_6_bytes(&self) -> Result<[u8; 6]>;
    fn deserialize_6_bytes(bytes: &[u8; 6]) -> Result<Self>;
}

impl SerializableDate for Date {
    fn serialize_2_bytes(&self) -> Result<[u8; 2]> {
        let components = self.datetime();
        let year = components.year();
        let month = components.month();
        let day = components.day();

        let yy = year - 2023;
        if !(0..128).contains(&yy) {
            bail!("Year out of range");
        }
        if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            bail!("Invalid month or day");
        }

        let value = ((yy as u16) << 9) | ((month as u16) << 5) | (day as u16);
        Ok(value.to_be_bytes())
    }

    fn deserialize_2_bytes(bytes: &[u8; 2]) -> Result<Self> {
        let value = u16::from_be_bytes(*bytes);
        let day = (value & 0b11111) as u32;
        let month = ((value >> 5) & 0b1111) as u32;
        let yy = ((value >> 9) & 0b1111111) as i32;
        let year = yy + 2023;

        if !(1..=12).contains(&month) || !range_of_days_in_month(year, month).contains(&day) {
            bail!("Invalid month or day");
        }

        let date = Utc.with_ymd_and_hms(year, month, day, 0, 0, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("Invalid date"))?;
        Ok(Date::from_datetime(date))
    }

    fn serialize_4_bytes(&self) -> Result<[u8; 4]> {
        let reference_date = Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0).single().unwrap();
        let duration = self.datetime() - reference_date;
        let seconds = duration.num_seconds();
        let n = u32::try_from(seconds).map_err(|_| anyhow::anyhow!("Date out of range"))?;
        Ok(n.to_be_bytes())
    }

    fn deserialize_4_bytes(bytes: &[u8; 4]) -> Result<Self> {
        let n = u32::from_be_bytes(*bytes);
        let reference_date = Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0).single().unwrap();
        let date = reference_date + chrono::Duration::seconds(n as i64);
        Ok(Date::from_datetime(date))
    }

    fn serialize_6_bytes(&self) -> Result<[u8; 6]> {
        let reference_date = Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0).single().unwrap();
        let duration = self.datetime() - reference_date;
        let milliseconds = duration.num_milliseconds();
        let n = u64::try_from(milliseconds).map_err(|_| anyhow::anyhow!("Date out of range"))?;

        if n > 0xe5940a78a7ff {
            bail!("Date exceeds maximum representable value");
        }

        let bytes = n.to_be_bytes();
        Ok(bytes[2..8].try_into().unwrap())
    }

    fn deserialize_6_bytes(bytes: &[u8; 6]) -> Result<Self> {
        let mut full_bytes = [0u8; 8];
        full_bytes[2..].copy_from_slice(bytes);
        let n = u64::from_be_bytes(full_bytes);

        if n > 0xe5940a78a7ff {
            bail!("Date exceeds maximum representable value");
        }

        let reference_date = Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0).single().unwrap();
        let date = reference_date + chrono::Duration::milliseconds(n as i64);
        Ok(Date::from_datetime(date))
    }
}

pub fn range_of_days_in_month(year: i32, month: u32) -> std::ops::Range<u32> {
    let next_month = if month == 12 {
        Utc.with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0).unwrap()
    } else {
        Utc.with_ymd_and_hms(year, month + 1, 1, 0, 0, 0).unwrap()
    };
    let last_day = (next_month - Duration::days(1)).day();
    1..last_day + 1
}

#[cfg(test)]
mod tests {
    use chrono::{ TimeZone, Timelike, Utc };
    use dcbor::Date;
    use super::SerializableDate;
    use hex_literal::hex;

    #[test]
    fn test_2_byte_dates() {
        // Base date serialization and deserialization
        let base_date = Date::from_datetime(Utc.with_ymd_and_hms(2023, 6, 20, 0, 0, 0).unwrap());
        let serialized = base_date.serialize_2_bytes().unwrap();
        assert_eq!(hex::encode(serialized), "00d4");
        let deserialized = Date::deserialize_2_bytes(&serialized).unwrap();
        assert_eq!(base_date, deserialized);

        // Minimum date
        let min_serialized = [0x00, 0x21];
        let min_date = Date::from_datetime(Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap());
        let deserialized_min = Date::deserialize_2_bytes(&min_serialized).unwrap();
        assert_eq!(min_date, deserialized_min);

        // Maximum date
        let max_serialized = [0xff, 0x9f];
        let deserialized_max = Date::deserialize_2_bytes(&max_serialized).unwrap();
        let expected_max_date = Date::from_datetime(
            Utc.with_ymd_and_hms(2150, 12, 31, 0, 0, 0).unwrap()
        );
        assert_eq!(deserialized_max, expected_max_date);

        // Invalid date
        let invalid_serialized = [0x00, 0x5e]; // Represents 2023-02-30, which is invalid
        assert!(Date::deserialize_2_bytes(&invalid_serialized).is_err());
    }

    #[test]
    fn test_4_byte_dates() {
        // Base date serialization and deserialization
        let base_date = Date::from_datetime(Utc.with_ymd_and_hms(2023, 6, 20, 12, 34, 56).unwrap());
        let serialized = base_date.serialize_4_bytes().unwrap();
        assert_eq!(serialized, hex!("2a41d470"));
        let deserialized = Date::deserialize_4_bytes(&serialized).unwrap();
        assert_eq!(base_date, deserialized);

        // Minimum date
        let min_serialized = hex!("00000000");
        let min_date = Date::from_datetime(Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0).unwrap());
        let deserialized_min = Date::deserialize_4_bytes(&min_serialized).unwrap();
        assert_eq!(min_date, deserialized_min);

        // Maximum date
        let max_serialized = hex!("ffffffff");
        let deserialized_max = Date::deserialize_4_bytes(&max_serialized).unwrap();
        let expected_max_date = Date::from_datetime(
            Utc.with_ymd_and_hms(2137, 2, 7, 6, 28, 15).unwrap()
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
                .unwrap()
        );
        let serialized = base_date.serialize_6_bytes().unwrap();
        assert_eq!(serialized, hex!("00a51125d895"));
        let deserialized = Date::deserialize_6_bytes(&serialized).unwrap();
        assert_eq!(base_date, deserialized);

        // Minimum date
        let min_serialized = hex!("000000000000");
        let min_date = Date::from_datetime(Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0).unwrap());
        let deserialized_min = Date::deserialize_6_bytes(&min_serialized).unwrap();
        assert_eq!(min_date, deserialized_min);

        // Maximum date
        let max_serialized = hex!("e5940a78a7ff");
        let deserialized_max = Date::deserialize_6_bytes(&max_serialized).unwrap();
        let expected_max_date = Date::from_datetime(
            Utc.with_ymd_and_hms(9999, 12, 31, 23, 59, 59)
                .unwrap()
                .with_nanosecond(999_000_000)
                .unwrap()
        );
        assert_eq!(deserialized_max, expected_max_date);

        // Invalid date (exceeds maximum representable value)
        let invalid_serialized = hex!("e5940a78a800");
        assert!(Date::deserialize_6_bytes(&invalid_serialized).is_err());
    }
}

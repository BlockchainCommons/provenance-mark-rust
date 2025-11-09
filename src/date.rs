use chrono::{Datelike, Duration, TimeZone, Utc};
use dcbor::prelude::*;

use crate::{Error, Result};

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
            return Err(Error::YearOutOfRange { year });
        }
        if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return Err(Error::InvalidMonthOrDay { year, month, day });
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

        if !(1..=12).contains(&month)
            || !range_of_days_in_month(year, month).contains(&day)
        {
            return Err(Error::InvalidMonthOrDay { year, month, day });
        }

        let date = Utc
            .with_ymd_and_hms(year, month, day, 0, 0, 0)
            .single()
            .ok_or_else(|| Error::InvalidDate {
                details: format!(
                    "Cannot construct date {year}-{month:02}-{day:02}"
                ),
            })?;
        Ok(Date::from_datetime(date))
    }

    fn serialize_4_bytes(&self) -> Result<[u8; 4]> {
        let reference_date =
            Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0).single().unwrap();
        let duration = self.datetime() - reference_date;
        let seconds = duration.num_seconds();
        let n = u32::try_from(seconds).map_err(|_| Error::DateOutOfRange {
            details: "seconds value too large for u32".to_string(),
        })?;
        Ok(n.to_be_bytes())
    }

    fn deserialize_4_bytes(bytes: &[u8; 4]) -> Result<Self> {
        let n = u32::from_be_bytes(*bytes);
        let reference_date =
            Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0).single().unwrap();
        let date = reference_date + chrono::Duration::seconds(n as i64);
        Ok(Date::from_datetime(date))
    }

    fn serialize_6_bytes(&self) -> Result<[u8; 6]> {
        let reference_date =
            Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0).single().unwrap();
        let duration = self.datetime() - reference_date;
        let milliseconds = duration.num_milliseconds();
        let n =
            u64::try_from(milliseconds).map_err(|_| Error::DateOutOfRange {
                details: "milliseconds value too large for u64".to_string(),
            })?;

        if n > 0xe5940a78a7ff {
            return Err(Error::DateOutOfRange {
                details: "date exceeds maximum representable value".to_string(),
            });
        }

        let bytes = n.to_be_bytes();
        Ok(bytes[2..8].try_into().unwrap())
    }

    fn deserialize_6_bytes(bytes: &[u8; 6]) -> Result<Self> {
        let mut full_bytes = [0u8; 8];
        full_bytes[2..].copy_from_slice(bytes);
        let n = u64::from_be_bytes(full_bytes);

        if n > 0xe5940a78a7ff {
            return Err(Error::DateOutOfRange {
                details: "date exceeds maximum representable value".to_string(),
            });
        }

        let reference_date =
            Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0).single().unwrap();
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

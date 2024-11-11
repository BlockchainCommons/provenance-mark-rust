use base64::Engine as _;
use dcbor::{prelude::*, Date};
use serde::ser::Serializer;
use serde::de::{ Deserializer, Error as DeError };
use serde::Deserialize;

pub fn serialize_base64<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    serializer.serialize_str(&encoded)
}

pub fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    base64::engine::general_purpose::STANDARD.decode(s).map_err(DeError::custom)
}

pub fn serialize_cbor<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    serialize_base64(bytes, serializer)
}

pub fn deserialize_cbor<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where D: Deserializer<'de>
{
    let data = deserialize_base64(deserializer)?;
    CBOR::try_from_data(&data).map_err(DeError::custom)?;
    Ok(data)
}

pub fn serialize_iso8601<S>(date: &Date, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&date.to_string())
}

pub fn deserialize_iso8601<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Date::from_string(s).map_err(serde::de::Error::custom)
}

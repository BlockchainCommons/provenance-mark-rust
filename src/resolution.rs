use std::{
    convert::TryFrom,
    ops::{Range, RangeFrom},
};

use dcbor::{Date, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{Error, Result, date::SerializableDate};

// LOW (16 bytes)
// 0000  0000  0000  00  00
// 0123  4567  89ab  cd  ef
// key   hash  id    seq date

// MEDIUM (32 bytes)
// 00000000  00000000  11111111  1111  1111
// 01234567  89abcdef  01234567  89ab  cdef
// key       hash      id        seq   date

// QUARTILE (58 bytes)
// 0000000000000000  1111111111111111  2222222222222222  3333  333333
// 0123456789abcdef  0123456789abcdef  0123456789abcdef  0123  456789
// key               hash              id                seq   date

// HIGH (106 bytes)
// 00000000000000001111111111111111  22222222222222223333333333333333
// 44444444444444445555555555555555  6666  666666
// 0123456789abcdef0123456789abcdef  0123456789abcdef0123456789abcdef
// 0123456789abcdef0123456789abcdef  0123  456789 key
// hash                              id                                seq
// date

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
#[serde(into = "u8", try_from = "u8")]
pub enum ProvenanceMarkResolution {
    Low      = 0,
    Medium   = 1,
    Quartile = 2,
    High     = 3,
}

impl From<ProvenanceMarkResolution> for u8 {
    fn from(res: ProvenanceMarkResolution) -> Self { res as u8 }
}

impl TryFrom<u8> for ProvenanceMarkResolution {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(ProvenanceMarkResolution::Low),
            1 => Ok(ProvenanceMarkResolution::Medium),
            2 => Ok(ProvenanceMarkResolution::Quartile),
            3 => Ok(ProvenanceMarkResolution::High),
            _ => Err(Error::ResolutionError {
                details: format!(
                    "invalid provenance mark resolution value: {}",
                    value
                ),
            }),
        }
    }
}

impl From<ProvenanceMarkResolution> for CBOR {
    fn from(res: ProvenanceMarkResolution) -> Self { CBOR::from(res as u8) }
}

impl TryFrom<CBOR> for ProvenanceMarkResolution {
    type Error = dcbor::Error;

    fn try_from(cbor: CBOR) -> dcbor::Result<Self> {
        let value: u8 = cbor.try_into()?;
        ProvenanceMarkResolution::try_from(value).map_err(dcbor::Error::from)
    }
}

type Res = ProvenanceMarkResolution;

impl ProvenanceMarkResolution {
    pub fn link_length(&self) -> usize {
        match self {
            Res::Low => 4,
            Res::Medium => 8,
            Res::Quartile => 16,
            Res::High => 32,
        }
    }

    pub fn seq_bytes_length(&self) -> usize {
        match self {
            Res::Low => 2,
            Res::Medium | Res::Quartile | Res::High => 4,
        }
    }

    pub fn date_bytes_length(&self) -> usize {
        match self {
            Res::Low => 2,
            Res::Medium => 4,
            Res::Quartile | Res::High => 6,
        }
    }

    pub fn fixed_length(&self) -> usize {
        self.link_length() * 3
            + self.seq_bytes_length()
            + self.date_bytes_length()
    }

    pub fn key_range(&self) -> Range<usize> { 0..self.link_length() }

    pub fn chain_id_range(&self) -> Range<usize> { 0..self.link_length() }

    pub fn hash_range(&self) -> Range<usize> {
        self.chain_id_range().end
            ..self.chain_id_range().end + self.link_length()
    }

    pub fn seq_bytes_range(&self) -> Range<usize> {
        self.hash_range().end..self.hash_range().end + self.seq_bytes_length()
    }

    pub fn date_bytes_range(&self) -> Range<usize> {
        self.seq_bytes_range().end
            ..self.seq_bytes_range().end + self.date_bytes_length()
    }

    pub fn info_range(&self) -> RangeFrom<usize> {
        self.date_bytes_range().end..
    }

    /// Serializes a Date into bytes based on the resolution.
    pub fn serialize_date(&self, date: Date) -> Result<Vec<u8>> {
        match self {
            Res::Low => date.serialize_2_bytes().map(|bytes| bytes.to_vec()),
            Res::Medium => date.serialize_4_bytes().map(|bytes| bytes.to_vec()),
            Res::Quartile | Res::High => {
                date.serialize_6_bytes().map(|bytes| bytes.to_vec())
            }
        }
    }

    /// Deserializes bytes into a Date based on the resolution.
    pub fn deserialize_date(&self, data: &[u8]) -> Result<Date> {
        match self {
            Res::Low if data.len() == 2 => {
                Date::deserialize_2_bytes(&[data[0], data[1]])
            }
            Res::Medium if data.len() == 4 => {
                Date::deserialize_4_bytes(&[data[0], data[1], data[2], data[3]])
            }
            Res::Quartile | Res::High if data.len() == 6 => {
                Date::deserialize_6_bytes(&[
                    data[0], data[1], data[2], data[3], data[4], data[5],
                ])
            }
            _ => Err(Error::ResolutionError {
                details: format!(
                    "invalid date length: expected 2, 4, or 6 bytes, got {}",
                    data.len()
                ),
            }),
        }
    }

    /// Serializes a sequence number into bytes based on the resolution.
    pub fn serialize_seq(&self, seq: u32) -> Result<Vec<u8>> {
        match self.seq_bytes_length() {
            2 => {
                if seq > (u16::MAX as u32) {
                    return Err(Error::ResolutionError {
                        details: format!(
                            "sequence number {} out of range for 2-byte format (max {})",
                            seq,
                            u16::MAX
                        ),
                    });
                }
                Ok((seq as u16).to_be_bytes().to_vec())
            }
            4 => Ok(seq.to_be_bytes().to_vec()),
            _ => unreachable!(),
        }
    }

    /// Deserializes bytes into a sequence number based on the resolution.
    pub fn deserialize_seq(&self, data: &[u8]) -> Result<u32> {
        match self.seq_bytes_length() {
            2 if data.len() == 2 => {
                Ok(u32::from(u16::from_be_bytes([data[0], data[1]])))
            }
            4 if data.len() == 4 => {
                Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
            }
            _ => Err(Error::ResolutionError {
                details: format!(
                    "invalid sequence number length: expected 2 or 4 bytes, got {}",
                    data.len()
                ),
            }),
        }
    }
}

impl std::fmt::Display for ProvenanceMarkResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Res::Low => write!(f, "low"),
            Res::Medium => write!(f, "medium"),
            Res::Quartile => write!(f, "quartile"),
            Res::High => write!(f, "high"),
        }
    }
}

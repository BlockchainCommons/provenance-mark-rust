use serde::{ Serialize, Deserialize };
use crate::util::{ serialize_seed, deserialize_seed };

pub const PROVENANCE_SEED_LENGTH: usize = 32;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProvenanceSeed(
    #[serde(serialize_with = "serialize_seed", deserialize_with = "deserialize_seed")]
    [u8; PROVENANCE_SEED_LENGTH]
);

impl ProvenanceSeed {
    pub fn to_bytes(&self) -> [u8; PROVENANCE_SEED_LENGTH] {
        self.0
    }

    pub fn from_bytes(bytes: [u8; PROVENANCE_SEED_LENGTH]) -> Self {
        Self(bytes)
    }

    pub fn hex(&self) -> String {
        hex::encode(self.0)
    }
}

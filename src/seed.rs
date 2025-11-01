use bc_rand::{
    RandomNumberGenerator, SecureRandomNumberGenerator, rng_random_data,
};
use serde::{Deserialize, Serialize};
use dcbor::prelude::*;

use crate::{crypto_utils::extend_key, util::{deserialize_block, serialize_block}, Error, Result};

pub const PROVENANCE_SEED_LENGTH: usize = 32;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProvenanceSeed(
    #[serde(
        serialize_with = "serialize_block",
        deserialize_with = "deserialize_block"
    )]
    [u8; PROVENANCE_SEED_LENGTH],
);

impl ProvenanceSeed {
    pub fn new() -> Self {
        let mut rng = SecureRandomNumberGenerator;
        Self::new_using(&mut rng)
    }

    pub fn new_using(rng: &mut impl RandomNumberGenerator) -> Self {
        // Randomness for a new seed can come from any secure random number
        // generator.
        let data = rng_random_data(rng, PROVENANCE_SEED_LENGTH);
        let mut seed_data = [0; PROVENANCE_SEED_LENGTH];
        seed_data.copy_from_slice(&data);
        Self::from_bytes(seed_data)
    }

    pub fn new_with_passphrase(passphrase: &str) -> Self {
        let seed_data = extend_key(passphrase.as_bytes());
        Self::from_bytes(seed_data)
    }

    pub fn to_bytes(&self) -> [u8; PROVENANCE_SEED_LENGTH] { self.0 }

    pub fn from_bytes(bytes: [u8; PROVENANCE_SEED_LENGTH]) -> Self {
        Self(bytes)
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != PROVENANCE_SEED_LENGTH {
            return Err(Error::InvalidSeedLength {
                actual: bytes.len(),
            });
        }
        let mut seed_bytes = [0u8; PROVENANCE_SEED_LENGTH];
        seed_bytes.copy_from_slice(bytes);
        Ok(Self::from_bytes(seed_bytes))
    }

    pub fn hex(&self) -> String { hex::encode(self.0) }
}

impl Default for ProvenanceSeed {
    fn default() -> Self { Self::new() }
}

impl From<[u8; PROVENANCE_SEED_LENGTH]> for ProvenanceSeed {
    fn from(bytes: [u8; PROVENANCE_SEED_LENGTH]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<ProvenanceSeed> for [u8; PROVENANCE_SEED_LENGTH] {
    fn from(seed: ProvenanceSeed) -> Self { seed.to_bytes() }
}

impl From<ProvenanceSeed> for CBOR {
    fn from(seed: ProvenanceSeed) -> Self {
        CBOR::to_byte_string(seed.to_bytes())
    }
}

impl TryFrom<CBOR> for ProvenanceSeed {
    type Error = dcbor::Error;

    fn try_from(cbor: CBOR) -> dcbor::Result<Self> {
        let bytes: Vec<u8> = cbor.try_byte_string()?;
        ProvenanceSeed::from_slice(&bytes)
            .map_err(|e| dcbor::Error::Custom(e.to_string()))
    }
}

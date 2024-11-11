use bc_rand::{ rng_random_data, RandomNumberGenerator, SecureRandomNumberGenerator };
use serde::{ Serialize, Deserialize };
use crate::util::{ serialize_block, deserialize_block };

pub const PROVENANCE_SEED_LENGTH: usize = 32;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProvenanceSeed(
    #[serde(serialize_with = "serialize_block", deserialize_with = "deserialize_block")]
    [u8; PROVENANCE_SEED_LENGTH]
);

impl ProvenanceSeed {
    pub fn new() -> Self {
        let mut rng = SecureRandomNumberGenerator;
        Self::new_using(&mut rng)
    }

    pub fn new_using(rng: &mut impl RandomNumberGenerator) -> Self {
        // Randomness for a new seed can come from any secure random number generator.
        let data = rng_random_data(rng, PROVENANCE_SEED_LENGTH);
        let mut seed_data = [0; PROVENANCE_SEED_LENGTH];
        seed_data.copy_from_slice(&data);
        Self::from_bytes(seed_data)
    }

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

impl Default for ProvenanceSeed {
    fn default() -> Self {
        Self::new()
    }
}

impl From<[u8; PROVENANCE_SEED_LENGTH]> for ProvenanceSeed {
    fn from(bytes: [u8; PROVENANCE_SEED_LENGTH]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<ProvenanceSeed> for [u8; PROVENANCE_SEED_LENGTH] {
    fn from(seed: ProvenanceSeed) -> Self {
        seed.to_bytes()
    }
}

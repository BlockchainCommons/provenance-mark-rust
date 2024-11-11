use serde::{ Serialize, Deserialize };
use crate::util::{ serialize_block, deserialize_block };

pub const RNG_STATE_LENGTH: usize = 32;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RngState(
    #[serde(serialize_with = "serialize_block", deserialize_with = "deserialize_block")] [
        u8;
        RNG_STATE_LENGTH
    ],
);

impl RngState {
    pub fn to_bytes(&self) -> [u8; RNG_STATE_LENGTH] {
        self.0
    }

    pub fn from_bytes(bytes: [u8; RNG_STATE_LENGTH]) -> Self {
        Self(bytes)
    }

    pub fn hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl From<[u8; RNG_STATE_LENGTH]> for RngState {
    fn from(bytes: [u8; RNG_STATE_LENGTH]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<RngState> for [u8; RNG_STATE_LENGTH] {
    fn from(state: RngState) -> Self {
        state.to_bytes()
    }
}

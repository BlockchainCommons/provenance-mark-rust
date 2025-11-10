use dcbor::prelude::*;
use serde::{Deserialize, Serialize};

use crate::util::{deserialize_block, serialize_block};

pub const RNG_STATE_LENGTH: usize = 32;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RngState(
    #[serde(
        serialize_with = "serialize_block",
        deserialize_with = "deserialize_block"
    )]
    [u8; RNG_STATE_LENGTH],
);

impl RngState {
    pub fn to_bytes(&self) -> [u8; RNG_STATE_LENGTH] { self.0 }

    pub fn from_bytes(bytes: [u8; RNG_STATE_LENGTH]) -> Self { Self(bytes) }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != RNG_STATE_LENGTH {
            return Err(format!(
                "invalid RNG state length: expected {} bytes, got {} bytes",
                RNG_STATE_LENGTH,
                bytes.len()
            ));
        }
        let mut state_bytes = [0u8; RNG_STATE_LENGTH];
        state_bytes.copy_from_slice(bytes);
        Ok(Self::from_bytes(state_bytes))
    }

    pub fn hex(&self) -> String { hex::encode(self.0) }
}

impl From<[u8; RNG_STATE_LENGTH]> for RngState {
    fn from(bytes: [u8; RNG_STATE_LENGTH]) -> Self { Self::from_bytes(bytes) }
}

impl From<RngState> for [u8; RNG_STATE_LENGTH] {
    fn from(state: RngState) -> Self { state.to_bytes() }
}

impl From<RngState> for CBOR {
    fn from(state: RngState) -> Self {
        CBOR::to_byte_string(state.to_bytes().as_ref())
    }
}

impl TryFrom<CBOR> for RngState {
    type Error = dcbor::Error;

    fn try_from(cbor: CBOR) -> dcbor::Result<Self> {
        let bytes: Vec<u8> = cbor.try_byte_string()?;
        RngState::from_slice(&bytes)
            .map_err(|e| dcbor::Error::Custom(e.to_string()))
    }
}

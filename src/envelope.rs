use bc_envelope::prelude::*;

use crate::{Error, ProvenanceMark, Result};

impl EnvelopeEncodable for ProvenanceMark {
    fn into_envelope(self) -> Envelope { Envelope::new(self.to_cbor()) }
}

impl TryFrom<Envelope> for ProvenanceMark {
    type Error = Error;

    fn try_from(envelope: Envelope) -> Result<Self> {
        let leaf = envelope.subject().try_leaf().map_err(|e| {
            Error::Cbor(dcbor::Error::Custom(format!("envelope error: {}", e)))
        })?;
        let cbor_result: std::result::Result<Self, dcbor::Error> =
            leaf.try_into();
        cbor_result.map_err(Error::Cbor)
    }
}

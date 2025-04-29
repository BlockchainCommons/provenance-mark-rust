use bc_envelope::prelude::*;
use anyhow::{Error, Result};

use crate::ProvenanceMark;

impl EnvelopeEncodable for ProvenanceMark {
    fn into_envelope(self) -> Envelope {
        Envelope::new(self.to_cbor())
    }
}

impl TryFrom<Envelope> for ProvenanceMark {
    type Error = Error;

    fn try_from(envelope: Envelope) -> Result<Self> {
        Ok(envelope.subject().try_leaf()?.try_into()?)
    }
}

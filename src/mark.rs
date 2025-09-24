#[cfg(feature = "envelope")]
use std::sync::Arc;

#[cfg(feature = "envelope")]
use bc_envelope::{FormatContext, with_format_context_mut};
use bc_ur::bytewords;
// use bc_tags;
use dcbor::{Date, prelude::*};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    Error, ProvenanceMarkResolution, Result,
    crypto_utils::{SHA256_SIZE, obfuscate, sha256, sha256_prefix},
    util::{
        deserialize_base64, deserialize_cbor, deserialize_iso8601,
        serialize_base64, serialize_cbor, serialize_iso8601,
    },
};

// JSON Example:
// {"chainID":"znwVmQ==","date":"2023-06-20T00:00:00Z","hash":"ZaTfvw==","key":"
// znwVmQ==","res":0,"seq":0}

#[derive(Serialize, Clone)]
pub struct ProvenanceMark {
    seq: u32,

    #[serde(serialize_with = "serialize_iso8601")]
    date: Date,

    res: ProvenanceMarkResolution,

    #[serde(serialize_with = "serialize_base64")]
    chain_id: Vec<u8>,

    #[serde(serialize_with = "serialize_base64")]
    key: Vec<u8>,

    #[serde(serialize_with = "serialize_base64")]
    hash: Vec<u8>,

    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_cbor"
    )]
    info_bytes: Vec<u8>,

    #[serde(skip)]
    seq_bytes: Vec<u8>,

    #[serde(skip)]
    date_bytes: Vec<u8>,
}

impl<'de> Deserialize<'de> for ProvenanceMark {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ProvenanceMarkHelper {
            res: ProvenanceMarkResolution,
            #[serde(deserialize_with = "deserialize_base64")]
            key: Vec<u8>,
            #[serde(deserialize_with = "deserialize_base64")]
            hash: Vec<u8>,
            #[serde(deserialize_with = "deserialize_base64")]
            chain_id: Vec<u8>,
            #[serde(default, deserialize_with = "deserialize_cbor")]
            info_bytes: Vec<u8>,
            seq: u32,
            #[serde(deserialize_with = "deserialize_iso8601")]
            date: Date,
        }

        let helper = ProvenanceMarkHelper::deserialize(deserializer)?;
        let seq_bytes = helper
            .res
            .serialize_seq(helper.seq)
            .map_err(serde::de::Error::custom)?;
        let date_bytes = helper
            .res
            .serialize_date(helper.date.clone())
            .map_err(serde::de::Error::custom)?;

        Ok(ProvenanceMark {
            res: helper.res,
            key: helper.key,
            hash: helper.hash,
            chain_id: helper.chain_id,
            seq_bytes,
            date_bytes,
            info_bytes: helper.info_bytes,
            seq: helper.seq,
            date: helper.date,
        })
    }
}

impl PartialEq for ProvenanceMark {
    fn eq(&self, other: &Self) -> bool {
        self.res == other.res && self.message() == other.message()
    }
}

impl Eq for ProvenanceMark {}

impl std::hash::Hash for ProvenanceMark {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.res.hash(state);
        self.message().hash(state);
    }
}

impl ProvenanceMark {
    pub fn res(&self) -> ProvenanceMarkResolution { self.res }
    pub fn key(&self) -> &[u8] { &self.key }
    pub fn hash(&self) -> &[u8] { &self.hash }
    pub fn chain_id(&self) -> &[u8] { &self.chain_id }
    pub fn seq_bytes(&self) -> &[u8] { &self.seq_bytes }
    pub fn date_bytes(&self) -> &[u8] { &self.date_bytes }

    pub fn seq(&self) -> u32 { self.seq }
    pub fn date(&self) -> &Date { &self.date }

    pub fn message(&self) -> Vec<u8> {
        let payload = [
            self.chain_id.clone(),
            self.hash.clone(),
            self.seq_bytes.clone(),
            self.date_bytes.clone(),
            self.info_bytes.clone(),
        ]
        .concat();
        [self.key.clone(), obfuscate(&self.key, payload)].concat()
    }

    pub fn info(&self) -> Option<CBOR> {
        if self.info_bytes.is_empty() {
            None
        } else {
            CBOR::try_from_data(&self.info_bytes).unwrap().into()
        }
    }
}

impl ProvenanceMark {
    pub fn new(
        res: ProvenanceMarkResolution,
        key: Vec<u8>,
        next_key: Vec<u8>,
        chain_id: Vec<u8>,
        seq: u32,
        date: Date,
        info: Option<impl CBOREncodable>,
    ) -> Result<Self> {
        if key.len() != res.link_length() {
            return Err(Error::InvalidKeyLength {
                expected: res.link_length(),
                actual: key.len(),
            });
        }
        if next_key.len() != res.link_length() {
            return Err(Error::InvalidNextKeyLength {
                expected: res.link_length(),
                actual: next_key.len(),
            });
        }
        if chain_id.len() != res.link_length() {
            return Err(Error::InvalidChainIdLength {
                expected: res.link_length(),
                actual: chain_id.len(),
            });
        }

        let date_bytes = res.serialize_date(date)?;
        let seq_bytes = res.serialize_seq(seq)?;

        let date = res.deserialize_date(&date_bytes)?;

        let info_bytes = match info {
            Some(info) => info.to_cbor_data(),
            None => Vec::new(),
        };

        let hash = Self::make_hash(
            res,
            &key,
            next_key,
            &chain_id,
            &seq_bytes,
            &date_bytes,
            &info_bytes,
        );

        Ok(Self {
            res,
            key,
            hash,
            chain_id,
            seq_bytes,
            date_bytes,
            info_bytes,

            seq,
            date,
        })
    }

    pub fn from_message(
        res: ProvenanceMarkResolution,
        message: Vec<u8>,
    ) -> Result<Self> {
        if message.len() < res.fixed_length() {
            return Err(Error::InvalidMessageLength {
                expected: res.fixed_length(),
                actual: message.len(),
            });
        }

        let key = message[res.key_range()].to_vec();
        let payload = obfuscate(&key, &message[res.link_length()..]);
        let hash = payload[res.hash_range()].to_vec();
        let chain_id = payload[res.chain_id_range()].to_vec();
        let seq_bytes = payload[res.seq_bytes_range()].to_vec();
        let seq = res.deserialize_seq(&seq_bytes)?;
        let date_bytes = payload[res.date_bytes_range()].to_vec();
        let date = res.deserialize_date(&date_bytes)?;

        let info_bytes = payload[res.info_range()].to_vec();
        if !info_bytes.is_empty() && CBOR::try_from_data(&info_bytes).is_err() {
            return Err(Error::InvalidInfoCbor);
        }
        Ok(Self {
            res,
            key,
            hash,
            chain_id,
            seq_bytes,
            date_bytes,
            info_bytes,

            seq,
            date,
        })
    }

    fn make_hash(
        res: ProvenanceMarkResolution,
        key: impl AsRef<[u8]>,
        next_key: impl AsRef<[u8]>,
        chain_id: impl AsRef<[u8]>,
        seq_bytes: impl AsRef<[u8]>,
        date_bytes: impl AsRef<[u8]>,
        info_bytes: impl AsRef<[u8]>,
    ) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(key.as_ref());
        buf.extend_from_slice(next_key.as_ref());
        buf.extend_from_slice(chain_id.as_ref());
        buf.extend_from_slice(seq_bytes.as_ref());
        buf.extend_from_slice(date_bytes.as_ref());
        buf.extend_from_slice(info_bytes.as_ref());

        sha256_prefix(&buf, res.link_length())
    }
}

impl ProvenanceMark {
    /// The first four bytes of the mark's hash as a hex string.
    pub fn identifier(&self) -> String { hex::encode(&self.hash[..4]) }

    /// The first four bytes of the mark's hash as upper-case ByteWords.
    pub fn bytewords_identifier(&self, prefix: bool) -> String {
        let s = bytewords::identifier(&self.hash[..4].try_into().unwrap())
            .to_uppercase();
        if prefix { format!("🅟 {}", s) } else { s }
    }

    /// The first four bytes of the mark's hash as Bytemoji.
    pub fn bytemoji_identifier(&self, prefix: bool) -> String {
        let s =
            bytewords::bytemoji_identifier(&self.hash[..4].try_into().unwrap())
                .to_uppercase();
        if prefix { format!("🅟 {}", s) } else { s }
    }
}

impl ProvenanceMark {
    pub fn precedes(&self, next: &ProvenanceMark) -> bool {
        // `next` can't be a genesis
        next.seq != 0 &&
            next.key != next.chain_id &&
            // `next` must have the next highest sequence number
            self.seq == next.seq - 1 &&
            // `next` must have an equal or later date
            self.date <= next.date &&
            // `next` must reveal the key that was used to generate this mark's hash
            self.hash ==
                Self::make_hash(
                    self.res,
                    &self.key,
                    &next.key,
                    &self.chain_id,
                    &self.seq_bytes,
                    &self.date_bytes,
                    &self.info_bytes
                )
    }

    pub fn is_sequence_valid(marks: &[ProvenanceMark]) -> bool {
        if marks.len() < 2 {
            return false;
        }
        if marks[0].seq == 0 && !marks[0].is_genesis() {
            return false;
        }
        marks.windows(2).all(|pair| pair[0].precedes(&pair[1]))
    }

    pub fn is_genesis(&self) -> bool {
        self.seq == 0 && self.key == self.chain_id
    }
}

impl ProvenanceMark {
    pub fn to_bytewords_with_style(&self, style: bytewords::Style) -> String {
        bytewords::encode(self.message(), style)
    }

    pub fn to_bytewords(&self) -> String {
        self.to_bytewords_with_style(bytewords::Style::Standard)
    }

    pub fn from_bytewords(
        res: ProvenanceMarkResolution,
        bytewords: &str,
    ) -> Result<Self> {
        let message = bytewords::decode(bytewords, bytewords::Style::Standard)?;
        Self::from_message(res, message)
    }
}

impl ProvenanceMark {
    pub fn to_url_encoding(&self) -> String {
        bytewords::encode(self.to_cbor_data(), bytewords::Style::Minimal)
    }

    pub fn from_url_encoding(url_encoding: &str) -> Result<Self> {
        let cbor_data =
            bytewords::decode(url_encoding, bytewords::Style::Minimal)?;
        let cbor = CBOR::try_from_data(cbor_data)?;
        Ok(Self::try_from(cbor)?)
    }
}

impl ProvenanceMark {
    // Example format:
    // ur:provenance/lfaegdtokebznlahftbsnlaxpsdiwecswsrnlsdsdpghrp
    pub fn to_url(&self, base: &str) -> Url {
        let mut url = Url::parse(base).unwrap();
        url.query_pairs_mut()
            .append_pair("provenance", &self.to_url_encoding());
        url
    }

    pub fn from_url(url: &Url) -> Result<Self> {
        let query = url.query_pairs().find(|(key, _)| key == "provenance");
        if let Some((_, value)) = query {
            Self::from_url_encoding(&value)
        } else {
            Err(Error::MissingUrlParameter {
                parameter: "provenance".to_string(),
            })
        }
    }
}

impl std::fmt::Debug for ProvenanceMark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut components = vec![
            format!("key: {}", hex::encode(&self.key)),
            format!("hash: {}", hex::encode(&self.hash)),
            format!("chainID: {}", hex::encode(&self.chain_id)),
            format!("seq: {}", self.seq),
            format!("date: {}", self.date.to_string()),
        ];

        if let Some(info) = self.info() {
            components.push(format!("info: {}", info.diagnostic()));
        }

        write!(f, "ProvenanceMark({})", components.join(", "))
    }
}

impl std::fmt::Display for ProvenanceMark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProvenanceMark({})", self.identifier())
    }
}

#[cfg(feature = "envelope")]
pub fn register_tags_in(context: &mut FormatContext) {
    bc_envelope::register_tags_in(context);

    context.tags_mut().set_summarizer(
        bc_tags::TAG_PROVENANCE_MARK,
        Arc::new(move |untagged_cbor: CBOR, _flat: bool| {
            let provenance_mark =
                ProvenanceMark::from_untagged_cbor(untagged_cbor)?;
            Ok(provenance_mark.to_string())
        }),
    );
}

#[cfg(feature = "envelope")]
pub fn register_tags() {
    with_format_context_mut!(|context: &mut FormatContext| {
        register_tags_in(context);
    });
}

impl CBORTagged for ProvenanceMark {
    fn cbor_tags() -> Vec<Tag> {
        tags_for_values(&[bc_tags::TAG_PROVENANCE_MARK])
    }
}

impl From<ProvenanceMark> for CBOR {
    fn from(value: ProvenanceMark) -> Self { value.tagged_cbor() }
}

impl CBORTaggedEncodable for ProvenanceMark {
    fn untagged_cbor(&self) -> CBOR {
        vec![self.res.to_cbor(), CBOR::to_byte_string(self.message())].to_cbor()
    }
}

impl TryFrom<CBOR> for ProvenanceMark {
    type Error = dcbor::Error;

    fn try_from(cbor: CBOR) -> dcbor::Result<Self> {
        Self::from_tagged_cbor(cbor)
    }
}

impl CBORTaggedDecodable for ProvenanceMark {
    fn from_untagged_cbor(cbor: CBOR) -> dcbor::Result<Self> {
        let v = CBOR::try_into_array(cbor)?;
        if v.len() != 2 {
            return Err("Invalid provenance mark length".into());
        }
        let res = ProvenanceMarkResolution::try_from(v[0].clone())?;
        let message = CBOR::try_into_byte_string(v[1].clone())?;
        Self::from_message(res, message).map_err(dcbor::Error::from)
    }
}

// Convert from an instance reference to an instance.
impl From<&ProvenanceMark> for ProvenanceMark {
    fn from(mark: &ProvenanceMark) -> Self { mark.clone() }
}

impl ProvenanceMark {
    pub fn fingerprint(&self) -> [u8; SHA256_SIZE] {
        sha256(self.to_cbor_data())
    }
}

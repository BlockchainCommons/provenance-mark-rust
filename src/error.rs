use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// Invalid key length for the given resolution
    #[error("invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    /// Invalid next key length for the given resolution
    #[error("invalid next key length: expected {expected}, got {actual}")]
    InvalidNextKeyLength { expected: usize, actual: usize },

    /// Invalid chain ID length for the given resolution
    #[error("invalid chain ID length: expected {expected}, got {actual}")]
    InvalidChainIdLength { expected: usize, actual: usize },

    /// Invalid message length for the given resolution
    #[error(
        "invalid message length: expected at least {expected}, got {actual}"
    )]
    InvalidMessageLength { expected: usize, actual: usize },

    /// Invalid CBOR data in info field
    #[error("invalid CBOR data in info field")]
    InvalidInfoCbor,

    /// Date out of range for serialization
    #[error("date out of range: {details}")]
    DateOutOfRange { details: String },

    /// Invalid date components
    #[error("invalid date: {details}")]
    InvalidDate { details: String },

    /// Missing required URL parameter
    #[error("missing required URL parameter: {parameter}")]
    MissingUrlParameter { parameter: String },

    /// Year out of range for 2-byte serialization
    #[error(
        "year out of range for 2-byte serialization: must be between 2023-2150, got {year}"
    )]
    YearOutOfRange { year: i32 },

    /// Invalid month or day
    #[error("invalid month ({month}) or day ({day}) for year {year}")]
    InvalidMonthOrDay { year: i32, month: u32, day: u32 },

    /// Resolution serialization error
    #[error("resolution serialization error: {details}")]
    ResolutionError { details: String },

    /// Bytewords encoding/decoding error
    #[error("bytewords error: {0}")]
    Bytewords(#[from] bc_ur::Error),

    /// CBOR encoding/decoding error
    #[error("CBOR error: {0}")]
    Cbor(#[from] dcbor::Error),

    /// URL parsing error
    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),

    /// Base64 decoding error
    #[error("base64 decoding error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Integer conversion error
    #[error("integer conversion error: {0}")]
    TryFromInt(#[from] std::num::TryFromIntError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<Error> for dcbor::Error {
    fn from(error: Error) -> dcbor::Error {
        match error {
            Error::Cbor(err) => err,
            _ => dcbor::Error::Custom(error.to_string()),
        }
    }
}

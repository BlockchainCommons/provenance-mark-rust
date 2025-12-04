#![doc(html_root_url = "https://docs.rs/provenance-mark/0.17.0")]
#![warn(rust_2018_idioms)]

//! # Introduction
//!
//! [Provenance Marks](https://provemark.com) provide a
//! cryptographically-secured system for establishing and verifying the
//! authenticity of works in an age of rampant AI-powered manipulation and
//! plagiarism. By combining cryptography, pseudorandom number generation, and
//! linguistic representation, this system generates unique, sequential marks
//! that commit to the content of preceding and subsequent works. These marks
//! ensure public and easy verification of provenance, offering robust security
//! and intuitive usability. Provenance Marks are particularly valuable for
//! securing artistic, intellectual, and commercial works against fraud and deep
//! fakes, protecting creators' reputations and the integrity of their
//! creations.
//!
//! # Getting Started
//!
//! ```toml
//! [dependencies]
//! provenance-mark = "0.17.0"
//! ```
//!
//! # Examples
//!
//! See the unit tests in the source code for examples of how to use this
//! library.

mod validate;
pub use validate::*;
mod error;
pub use error::{Error, Result};
mod resolution;
pub use resolution::*;
mod mark;
pub use mark::*;
mod mark_info;
pub use mark_info::*;
mod generator;
pub use generator::*;
mod seed;
pub use seed::*;
mod rng_state;
pub use rng_state::*;
pub mod crypto_utils;
pub mod date;
pub mod util;
pub mod xoshiro256starstar;

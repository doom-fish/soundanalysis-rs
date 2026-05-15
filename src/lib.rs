#![doc = include_str!("../README.md")]
//!
//! ---
//!
//! # API documentation
//!
//! Safe Rust bindings for Apple's
//! [SoundAnalysis](https://developer.apple.com/documentation/soundanalysis)
//! framework on macOS — on-device sound classification using the built-in
//! `version1` model (~300 everyday sounds).

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod error;
pub mod ffi;

#[cfg(feature = "analyze_file")]
#[cfg_attr(docsrs, doc(cfg(feature = "analyze_file")))]
pub mod classifier;

pub use error::SAError;

#[cfg(feature = "analyze_file")]
pub use classifier::{classify_file, known_classifications, Classification, ClassificationResult};

/// Common imports.
pub mod prelude {
    pub use crate::error::SAError;
    #[cfg(feature = "analyze_file")]
    pub use crate::classifier::{
        classify_file, known_classifications, Classification, ClassificationResult,
    };
}

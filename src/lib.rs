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
#![allow(
    clippy::borrow_as_ptr,
    clippy::elidable_lifetime_names,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::needless_pass_by_value
)]

pub mod error;
pub mod ffi;
pub mod observer;
pub mod request;
pub mod time;
pub(crate) mod utils;

#[cfg(feature = "analyze_file")]
#[cfg_attr(docsrs, doc(cfg(feature = "analyze_file")))]
pub mod classifier;

#[cfg(feature = "analyze_file")]
#[cfg_attr(docsrs, doc(cfg(feature = "analyze_file")))]
pub mod file;

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
pub mod live;

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
pub mod streaming;

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod async_api;

pub use error::{error_domain, ErrorCode, SAError, SNErrorCode, SNErrorDomain};
pub use observer::ResultsObserver as SNResultsObserving;
pub use observer::{AnalysisResult, ResultsObserver, ResultsObserverFns, SNResult};
pub use request::{
    AnalysisRequest, ClassifierIdentifier, ClassifySoundRequest, SNClassifierIdentifier,
    SNClassifySoundRequest, SNRequest,
};
pub use time::{TimeDurationConstraint, TimeDurationConstraintType, TimeRange};
pub use time::{TimeDurationConstraint as SNTimeDurationConstraint, TimeRange as SNTimeRange};

#[cfg(feature = "analyze_file")]
pub use classifier::{
    classify_file, classify_file_with_model, known_classifications, Classification,
    ClassificationResult,
};
#[cfg(feature = "analyze_file")]
pub use classifier::{
    Classification as SNClassification, ClassificationResult as SNClassificationResult,
};

#[cfg(feature = "analyze_file")]
pub use file::{AudioFileAnalyzer, AudioFileAnalyzer as SNAudioFileAnalyzer};

#[cfg(feature = "stream")]
pub use live::{start_live_classification, LiveClassification, StreamUpdate};

#[cfg(feature = "stream")]
pub use streaming::AudioStreamAnalyzer as SNAudioStreamAnalyzer;
#[cfg(feature = "stream")]
pub use streaming::{AudioStreamAnalyzer, AudioStreamFormat, PcmBuffer, PcmSampleFormat};

/// Common imports.
pub mod prelude {
    #[cfg(feature = "analyze_file")]
    pub use crate::classifier::{
        classify_file, classify_file_with_model, known_classifications, Classification,
        ClassificationResult,
    };
    #[cfg(feature = "analyze_file")]
    pub use crate::classifier::{
        Classification as SNClassification, ClassificationResult as SNClassificationResult,
    };
    pub use crate::error::{error_domain, ErrorCode, SAError, SNErrorCode, SNErrorDomain};
    #[cfg(feature = "analyze_file")]
    pub use crate::file::{AudioFileAnalyzer, AudioFileAnalyzer as SNAudioFileAnalyzer};
    #[cfg(feature = "stream")]
    pub use crate::live::{start_live_classification, LiveClassification, StreamUpdate};
    pub use crate::observer::ResultsObserver as SNResultsObserving;
    pub use crate::observer::{ResultsObserver, ResultsObserverFns, SNResult};
    pub use crate::request::{
        ClassifierIdentifier, ClassifySoundRequest, SNClassifierIdentifier, SNClassifySoundRequest,
        SNRequest,
    };
    #[cfg(feature = "stream")]
    pub use crate::streaming::AudioStreamAnalyzer as SNAudioStreamAnalyzer;
    #[cfg(feature = "stream")]
    pub use crate::streaming::{
        AudioStreamAnalyzer, AudioStreamFormat, PcmBuffer, PcmSampleFormat,
    };
    pub use crate::time::{TimeDurationConstraint, TimeDurationConstraintType, TimeRange};
    pub use crate::time::{
        TimeDurationConstraint as SNTimeDurationConstraint, TimeRange as SNTimeRange,
    };
}

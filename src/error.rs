//! Errors returned by the `SoundAnalysis` bridge.

use core::fmt;

use crate::ffi;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SAError {
    InvalidArgument(String),
    AudioLoadFailed(String),
    RequestCreateFailed(String),
    AnalysisFailed(String),
    Unknown { code: i32, message: String },
}

impl fmt::Display for SAError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(m) => write!(f, "invalid argument: {m}"),
            Self::AudioLoadFailed(m) => write!(f, "audio file load failed: {m}"),
            Self::RequestCreateFailed(m) => write!(f, "classifier init failed: {m}"),
            Self::AnalysisFailed(m) => write!(f, "sound analysis failed: {m}"),
            Self::Unknown { code, message } => write!(f, "soundanalysis error {code}: {message}"),
        }
    }
}

impl std::error::Error for SAError {}

pub(crate) unsafe fn from_swift(status: i32, error_str: *mut core::ffi::c_char) -> SAError {
    let message = if error_str.is_null() {
        String::new()
    } else {
        let s = core::ffi::CStr::from_ptr(error_str)
            .to_string_lossy()
            .into_owned();
        ffi::sa_string_free(error_str);
        s
    };
    match status {
        ffi::status::AUDIO_LOAD_FAILED => SAError::AudioLoadFailed(message),
        ffi::status::REQUEST_CREATE_FAILED => SAError::RequestCreateFailed(message),
        ffi::status::ANALYSIS_FAILED => SAError::AnalysisFailed(message),
        ffi::status::INVALID_ARGUMENT => SAError::InvalidArgument(message),
        code => SAError::Unknown { code, message },
    }
}

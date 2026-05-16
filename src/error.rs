//! Errors returned by the `SoundAnalysis` bridge.

use core::fmt;
use std::sync::OnceLock;

use crate::ffi;

static ERROR_DOMAIN: OnceLock<String> = OnceLock::new();

/// Rust mirror of `SoundAnalysis`'s `SNErrorCode` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(i32)]
pub enum ErrorCode {
    UnknownError = 1,
    OperationFailed = 2,
    InvalidFormat = 3,
    InvalidModel = 4,
    InvalidFile = 5,
}

impl ErrorCode {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            1 => Some(Self::UnknownError),
            2 => Some(Self::OperationFailed),
            3 => Some(Self::InvalidFormat),
            4 => Some(Self::InvalidModel),
            5 => Some(Self::InvalidFile),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_raw(self) -> i32 {
        self as i32
    }
}

impl From<ErrorCode> for i32 {
    fn from(value: ErrorCode) -> Self {
        value.as_raw()
    }
}

impl TryFrom<i32> for ErrorCode {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_raw(value).ok_or(value)
    }
}

/// Returns the framework's exported `SNErrorDomain` string.
#[must_use]
pub fn error_domain() -> &'static str {
    ERROR_DOMAIN.get_or_init(load_error_domain).as_str()
}

pub use error_domain as SNErrorDomain;
pub type SNErrorCode = ErrorCode;

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

fn load_error_domain() -> String {
    let domain_ptr = unsafe { ffi::sa_copy_sn_error_domain() };
    if domain_ptr.is_null() {
        return String::new();
    }

    let domain = unsafe { core::ffi::CStr::from_ptr(domain_ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::sa_string_free(domain_ptr) };
    domain
}

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

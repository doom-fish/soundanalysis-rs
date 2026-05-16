//! Raw FFI declarations matching the Swift bridge.

#![allow(missing_docs, non_camel_case_types)]

use core::ffi::{c_char, c_void};

#[repr(C)]
pub struct ClassificationRaw {
    pub identifier: *mut c_char,
    pub confidence: f64,
}

#[repr(C)]
pub struct ClassificationResultRaw {
    pub time_start: f64,
    pub time_duration: f64,
    pub classifications: *mut ClassificationRaw,
    pub classification_count: usize,
}

extern "C" {
    pub fn sa_string_free(s: *mut c_char);

    pub fn sa_known_classifications(
        out_array: *mut *mut *mut c_char,
        out_count: *mut usize,
    ) -> i32;
    pub fn sa_known_classifications_free(array: *mut *mut c_char, count: usize);

    pub fn sa_classify_file(
        audio_path: *const c_char,
        out_array: *mut *mut c_void,
        out_count: *mut usize,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_classification_results_free(array: *mut c_void, count: usize);

    pub fn sa_stream_start(
        callback: StreamCallback,
        user_info: *mut c_void,
        out_err: *mut *mut c_char,
    ) -> *mut c_void;

    pub fn sa_stream_stop(handle: *mut c_void);
}

pub type StreamCallback = unsafe extern "C" fn(
    user_info: *mut c_void,
    time_start: f64,
    time_duration: f64,
    classifications: *mut c_void,
    classification_count: usize,
);

pub mod status {
    pub const OK: i32 = 0;
    pub const INVALID_ARGUMENT: i32 = -1;
    pub const AUDIO_LOAD_FAILED: i32 = -2;
    pub const REQUEST_CREATE_FAILED: i32 = -3;
    pub const ANALYSIS_FAILED: i32 = -4;
    pub const UNKNOWN: i32 = -99;
}

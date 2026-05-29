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

#[repr(C)]
pub struct TimeDurationConstraintRaw {
    pub kind: i32,
    pub range_start_seconds: f64,
    pub range_duration_seconds: f64,
    pub values: *mut f64,
    pub value_count: usize,
}

impl TimeDurationConstraintRaw {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            kind: 0,
            range_start_seconds: 0.0,
            range_duration_seconds: 0.0,
            values: core::ptr::null_mut(),
            value_count: 0,
        }
    }
}

#[repr(C)]
pub struct StreamFormatRaw {
    pub sample_rate: f64,
    pub channel_count: u32,
    pub sample_format: i32,
    pub interleaved: bool,
}

#[repr(C)]
pub struct StreamBufferRaw {
    pub sample_format: i32,
    pub channel_count: u32,
    pub frame_length: usize,
    pub interleaved: bool,
    pub interleaved_data: *const c_void,
    pub planar_data: *const *const c_void,
}

// MARK: - ABI Layout Assertions
//
// The `#[repr(C)]` structs above cross the Rust <-> Swift `@_cdecl` FFI
// boundary either by value, via packed buffers, or by being read directly
// from a raw pointer (e.g. `ClassificationRaw` in the live trampoline). Their
// Swift counterparts live in
// `swift-bridge/Sources/SoundAnalysisBridge/Core.swift`
// (`SAClassificationRaw`, `SAClassificationResultRaw`,
// `SATimeDurationConstraintRaw`, `SAStreamFormatRaw`, `SAStreamBufferRaw`).
//
// These compile-time assertions pin the exact ABI shared with Swift: any change
// to a field type, field order, or padding fails the build immediately instead
// of silently corrupting marshalled data at runtime. The crate's MSRV (1.76)
// predates `offset_of!` (stable in 1.77), so we pin size and alignment only.
// If you change the layout here you MUST mirror it in Core.swift (and vice
// versa); the cross-language `sa_verify_ffi_layout` check in
// `tests/ffi_layout_tests.rs` guards that too.
use core::mem::{align_of, size_of};

const _: () = assert!(size_of::<ClassificationRaw>() == 16);
const _: () = assert!(align_of::<ClassificationRaw>() == 8);

const _: () = assert!(size_of::<ClassificationResultRaw>() == 32);
const _: () = assert!(align_of::<ClassificationResultRaw>() == 8);

const _: () = assert!(size_of::<TimeDurationConstraintRaw>() == 40);
const _: () = assert!(align_of::<TimeDurationConstraintRaw>() == 8);

const _: () = assert!(size_of::<StreamFormatRaw>() == 24);
const _: () = assert!(align_of::<StreamFormatRaw>() == 8);

const _: () = assert!(size_of::<StreamBufferRaw>() == 40);
const _: () = assert!(align_of::<StreamBufferRaw>() == 8);

pub type ObserverResultCallback = unsafe extern "C" fn(
    user_info: *mut c_void,
    time_start: f64,
    time_duration: f64,
    classifications: *mut c_void,
    classification_count: usize,
);

pub type ObserverErrorCallback =
    unsafe extern "C" fn(user_info: *mut c_void, status: i32, error_message: *mut c_char);

pub type ObserverCompleteCallback = unsafe extern "C" fn(user_info: *mut c_void);

extern "C" {
    pub fn sa_string_free(s: *mut c_char);
    pub fn sa_double_array_free(array: *mut f64, count: usize);
    pub fn sa_copy_sn_error_domain() -> *mut c_char;

    pub fn sa_known_classifications(out_array: *mut *mut *mut c_char, out_count: *mut usize)
        -> i32;
    pub fn sa_known_classifications_free(array: *mut *mut c_char, count: usize);

    pub fn sa_classify_file(
        audio_path: *const c_char,
        out_array: *mut *mut c_void,
        out_count: *mut usize,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_classification_results_free(array: *mut c_void, count: usize);

    pub fn sa_classify_file_with_model(
        audio_path: *const c_char,
        model_path: *const c_char,
        out_array: *mut *mut c_void,
        out_count: *mut usize,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sa_request_create_classifier(
        classifier: i32,
        out_request: *mut *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_request_create_model(
        model_path: *const c_char,
        out_request: *mut *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_request_retain(request: *mut c_void) -> *mut c_void;
    pub fn sa_request_release(request: *mut c_void);
    pub fn sa_request_get_overlap_factor(request: *mut c_void) -> f64;
    pub fn sa_request_set_overlap_factor(
        request: *mut c_void,
        overlap_factor: f64,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_request_get_window_duration(request: *mut c_void) -> f64;
    pub fn sa_request_set_window_duration(
        request: *mut c_void,
        seconds: f64,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_request_get_window_duration_constraint(
        request: *mut c_void,
        out_constraint: *mut TimeDurationConstraintRaw,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_request_known_classifications_for_request(
        request: *mut c_void,
        out_array: *mut *mut *mut c_char,
        out_count: *mut usize,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sa_time_duration_constraint_create_enumerated(
        durations: *const f64,
        count: usize,
        out_constraint: *mut TimeDurationConstraintRaw,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_time_duration_constraint_create_range(
        start_seconds: f64,
        duration_seconds: f64,
        out_constraint: *mut TimeDurationConstraintRaw,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sa_audio_file_analyzer_create(
        audio_path: *const c_char,
        out_analyzer: *mut *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_audio_file_analyzer_release(analyzer: *mut c_void);
    pub fn sa_audio_file_analyzer_add_request(
        analyzer: *mut c_void,
        request: *mut c_void,
        user_info: *mut c_void,
        result_callback: ObserverResultCallback,
        error_callback: ObserverErrorCallback,
        complete_callback: ObserverCompleteCallback,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_audio_file_analyzer_remove_request(analyzer: *mut c_void, request: *mut c_void);
    pub fn sa_audio_file_analyzer_remove_all_requests(analyzer: *mut c_void);
    pub fn sa_audio_file_analyzer_analyze(
        analyzer: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_audio_file_analyzer_analyze_with_completion(
        analyzer: *mut c_void,
        out_did_reach_end_of_file: *mut bool,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_audio_file_analyzer_cancel_analysis(analyzer: *mut c_void);
    pub fn sa_audio_file_analyzer_analyze_async(
        audio_path: *const c_char,
        cb: unsafe extern "C" fn(bool, *const i8, *mut c_void),
        ctx: *mut c_void,
    );

    pub fn sa_audio_stream_analyzer_create(
        format: *const c_void,
        out_analyzer: *mut *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_audio_stream_analyzer_release(analyzer: *mut c_void);
    pub fn sa_audio_stream_analyzer_add_request(
        analyzer: *mut c_void,
        request: *mut c_void,
        user_info: *mut c_void,
        result_callback: ObserverResultCallback,
        error_callback: ObserverErrorCallback,
        complete_callback: ObserverCompleteCallback,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_audio_stream_analyzer_remove_request(analyzer: *mut c_void, request: *mut c_void);
    pub fn sa_audio_stream_analyzer_remove_all_requests(analyzer: *mut c_void);
    pub fn sa_audio_stream_analyzer_analyze_audio_buffer(
        analyzer: *mut c_void,
        buffer: *const c_void,
        audio_frame_position: i64,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sa_audio_stream_analyzer_complete_analysis(analyzer: *mut c_void);

    pub fn sa_stream_start(
        callback: StreamCallback,
        user_info: *mut c_void,
        out_err: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn sa_stream_stop(handle: *mut c_void);

    /// Cross-language ABI check implemented in the Swift bridge.
    ///
    /// Returns `true` only if the Swift `MemoryLayout` (size, stride and
    /// alignment) of every FFI struct matches the values pinned on the Rust
    /// side. Verified by `tests/ffi_layout_tests.rs`.
    pub fn sa_verify_ffi_layout() -> bool;
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

pub mod classifier_identifier {
    pub const VERSION1: i32 = 1;
}

pub mod constraint_type {
    pub const ENUMERATED: i32 = 1;
    pub const RANGE: i32 = 2;
}

pub mod sample_format {
    pub const FLOAT32: i32 = 1;
    pub const FLOAT64: i32 = 2;
    pub const INT16: i32 = 3;
    pub const INT32: i32 = 4;
}

//! ABI layout assertions for the `#[repr(C)]` structs shared with the Swift bridge.
//!
//! These structs cross the Rust <-> Swift `@_cdecl` FFI boundary by value, via
//! packed buffers, or by being read directly from a raw pointer (e.g.
//! `ClassificationRaw` in the live trampoline). If their size or alignment ever
//! drifts from what the Swift side expects, the data marshalling silently
//! corrupts. These tests pin the layout so accidental field reordering / type
//! changes are caught at `cargo test` time rather than as runtime garbage.

use std::mem::{align_of, size_of};

use soundanalysis::ffi::{
    sa_verify_ffi_layout, ClassificationRaw, ClassificationResultRaw, StreamBufferRaw,
    StreamFormatRaw, TimeDurationConstraintRaw,
};

#[test]
fn classification_raw_layout() {
    // *mut c_char + f64
    assert_eq!(
        size_of::<ClassificationRaw>(),
        16,
        "ClassificationRaw size drifted"
    );
    assert_eq!(
        align_of::<ClassificationRaw>(),
        8,
        "ClassificationRaw alignment drifted"
    );
}

#[test]
fn classification_result_raw_layout() {
    // f64, f64, *mut, usize
    assert_eq!(
        size_of::<ClassificationResultRaw>(),
        32,
        "ClassificationResultRaw size drifted"
    );
    assert_eq!(
        align_of::<ClassificationResultRaw>(),
        8,
        "ClassificationResultRaw alignment drifted"
    );
}

#[test]
fn time_duration_constraint_raw_layout() {
    // i32 (pad), f64, f64, *mut, usize
    assert_eq!(
        size_of::<TimeDurationConstraintRaw>(),
        40,
        "TimeDurationConstraintRaw size drifted"
    );
    assert_eq!(
        align_of::<TimeDurationConstraintRaw>(),
        8,
        "TimeDurationConstraintRaw alignment drifted"
    );
}

#[test]
fn stream_format_raw_layout() {
    // f64, u32, i32, bool (trailing pad to 8)
    assert_eq!(
        size_of::<StreamFormatRaw>(),
        24,
        "StreamFormatRaw size drifted"
    );
    assert_eq!(
        align_of::<StreamFormatRaw>(),
        8,
        "StreamFormatRaw alignment drifted"
    );
}

#[test]
fn stream_buffer_raw_layout() {
    // i32, u32, usize, bool (pad), *const, *const *const
    assert_eq!(
        size_of::<StreamBufferRaw>(),
        40,
        "StreamBufferRaw size drifted"
    );
    assert_eq!(
        align_of::<StreamBufferRaw>(),
        8,
        "StreamBufferRaw alignment drifted"
    );
}

/// Cross-language ABI check: asks the Swift bridge to verify that *its*
/// `MemoryLayout` (stride/alignment) for all FFI structs matches the values
/// pinned on the Rust side. A `false` return means the Rust and Swift layouts
/// genuinely disagree, which is a real ABI bug.
#[test]
fn ffi_layout_matches_swift() {
    // SAFETY: `sa_verify_ffi_layout` takes no arguments and only reads
    // compile-time `MemoryLayout` constants in the Swift bridge.
    let matches = unsafe { sa_verify_ffi_layout() };
    assert!(
        matches,
        "Swift FFI struct layout disagrees with Rust layout (ABI mismatch)"
    );
}

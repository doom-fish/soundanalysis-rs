#![cfg(feature = "stream")]

use std::ffi::{c_void, CStr};
use std::ptr;

use soundanalysis::ffi;

fn analyze_raw(frame_length: usize, planes: &[*const c_void]) -> (i32, String) {
    let format = ffi::StreamFormatRaw {
        sample_rate: 16_000.0,
        channel_count: 1,
        sample_format: ffi::sample_format::FLOAT32,
        interleaved: false,
    };
    let mut analyzer = ptr::null_mut();
    let mut err = ptr::null_mut();
    let status = unsafe {
        ffi::sa_audio_stream_analyzer_create(
            ptr::from_ref(&format).cast(),
            &raw mut analyzer,
            &raw mut err,
        )
    };
    assert_eq!(status, ffi::status::OK);

    let buffer = ffi::StreamBufferRaw {
        sample_format: ffi::sample_format::FLOAT32,
        channel_count: 1,
        frame_length,
        interleaved: false,
        interleaved_data: ptr::null(),
        planar_data: planes.as_ptr(),
    };
    let status = unsafe {
        ffi::sa_audio_stream_analyzer_analyze_audio_buffer(
            analyzer,
            ptr::from_ref(&buffer).cast(),
            0,
            &raw mut err,
        )
    };
    let message = if err.is_null() {
        String::new()
    } else {
        let message = unsafe { CStr::from_ptr(err) }
            .to_string_lossy()
            .into_owned();
        unsafe { ffi::sa_string_free(err) };
        message
    };
    unsafe { ffi::sa_audio_stream_analyzer_release(analyzer) };
    (status, message)
}

#[test]
fn bridge_rejects_frame_counts_before_allocating_or_copying() {
    let sample = [0.0_f32; 4];
    let planes = [sample.as_ptr().cast::<c_void>()];

    let (status, message) = analyze_raw(1 << 30, &planes);
    assert_eq!(status, ffi::status::INVALID_ARGUMENT, "{message}");
    assert_eq!(message, "PCM buffer is too large for an AVAudioPCMBuffer");

    for frame_length in [0, usize::MAX] {
        let (status, message) = analyze_raw(frame_length, &planes);
        assert_eq!(status, ffi::status::INVALID_ARGUMENT, "{message}");
        assert_eq!(message, "PCM buffer must contain at least one frame");
    }

    let (status, message) = analyze_raw(sample.len(), &planes);
    assert_eq!(status, ffi::status::OK, "{message}");
}

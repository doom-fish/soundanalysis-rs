//! Live microphone sound classification via `SNAudioStreamAnalyzer`.

use core::ffi::c_void;
use std::sync::Arc;
use std::sync::Mutex;

use crate::classifier::Classification;
use crate::error::SAError;
use crate::ffi;

/// One streaming classification update — a time-bucketed list of
/// classification hypotheses, ordered most-confident first.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamUpdate {
    /// Seconds since the stream started.
    pub time_start: f64,
    /// Duration of the analysis window in seconds.
    pub time_duration: f64,
    pub classifications: Vec<Classification>,
}

/// RAII guard for a live mic classification session. Drop to stop
/// the engine.
#[allow(clippy::type_complexity)]
pub struct LiveClassification {
    handle: *mut c_void,
    _callback: Arc<Mutex<Box<dyn FnMut(StreamUpdate) + Send + 'static>>>,
}

unsafe impl Send for LiveClassification {}
unsafe impl Sync for LiveClassification {}

impl Drop for LiveClassification {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::sa_stream_stop(self.handle) };
            self.handle = core::ptr::null_mut();
        }
    }
}

type StreamCb = Mutex<Box<dyn FnMut(StreamUpdate) + Send + 'static>>;

unsafe extern "C" fn trampoline(
    user_info: *mut c_void,
    time_start: f64,
    time_duration: f64,
    classifications: *mut c_void,
    classification_count: usize,
) {
    if user_info.is_null() {
        return;
    }
    let cb_arc_ptr = user_info.cast::<StreamCb>();
    let typed = classifications.cast::<ffi::ClassificationRaw>();
    let mut classes = Vec::with_capacity(classification_count);
    if !typed.is_null() {
        for i in 0..classification_count {
            let raw = unsafe { &*typed.add(i) };
            let id = if raw.identifier.is_null() {
                String::new()
            } else {
                unsafe { core::ffi::CStr::from_ptr(raw.identifier) }
                    .to_string_lossy()
                    .into_owned()
            };
            classes.push(Classification {
                identifier: id,
                confidence: raw.confidence,
            });
        }
    }
    let Ok(mut guard) = (unsafe { &*cb_arc_ptr }).lock() else {
        return;
    };
    guard(StreamUpdate {
        time_start,
        time_duration,
        classifications: classes,
    });
}

/// Start live microphone classification. Returns a
/// [`LiveClassification`] guard that stops the engine when dropped.
///
/// # Errors
///
/// Returns [`SAError::AnalysisFailed`] if the Swift bridge can't
/// install the tap (e.g. mic permission denied, engine refuses to
/// start). The calling app must have the `NSMicrophoneUsageDescription`
/// Info.plist key and the user must have granted access.
pub fn start_live_classification<F>(callback: F) -> Result<LiveClassification, SAError>
where
    F: FnMut(StreamUpdate) + Send + 'static,
{
    let boxed: Box<dyn FnMut(StreamUpdate) + Send + 'static> = Box::new(callback);
    let arc: Arc<StreamCb> = Arc::new(Mutex::new(boxed));
    let raw = Arc::into_raw(arc.clone()).cast::<c_void>().cast_mut();
    let mut err_msg: *mut core::ffi::c_char = core::ptr::null_mut();
    let handle = unsafe { ffi::sa_stream_start(trampoline, raw, &mut err_msg) };
    if handle.is_null() {
        // Take back the Arc we leaked into raw so it can drop.
        unsafe { Arc::from_raw(raw.cast::<StreamCb>()) };
        let msg = if err_msg.is_null() {
            "stream start failed".to_string()
        } else {
            let m = unsafe { core::ffi::CStr::from_ptr(err_msg) }
                .to_string_lossy()
                .into_owned();
            unsafe { ffi::sa_string_free(err_msg) };
            m
        };
        return Err(SAError::AnalysisFailed(msg));
    }
    Ok(LiveClassification {
        handle,
        _callback: arc,
    })
}

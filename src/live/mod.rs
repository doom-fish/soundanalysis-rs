//! Live microphone sound classification via `SNAudioStreamAnalyzer`.

use core::ffi::{c_char, c_void};
use std::sync::{Mutex, PoisonError};

use doom_fish_utils::callback_context::CallbackContext;
use doom_fish_utils::panic_safe::catch_user_panic;

use crate::classifier::Classification;
use crate::error::{from_swift, SAError};
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

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum LiveEvent {
    Update(StreamUpdate),
    Failed(SAError),
    Completed,
}

type LiveHandler = Mutex<Box<dyn FnMut(LiveEvent) + Send + 'static>>;
type LiveContext = CallbackContext<LiveHandler>;

/// RAII guard for a live mic classification session. Drop to stop
/// the engine.
pub struct LiveClassification {
    handle: *mut c_void,
    _context: LiveContext,
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

unsafe fn deliver(context: *mut c_void, site: &str, event: LiveEvent) {
    let _ = unsafe {
        LiveContext::with(context, site, |handler| {
            let mut handler = handler.lock().unwrap_or_else(PoisonError::into_inner);
            handler(event);
        })
    };
}

unsafe extern "C" fn result_trampoline(
    user_info: *mut c_void,
    time_start: f64,
    time_duration: f64,
    classifications: *mut c_void,
    classification_count: usize,
) {
    catch_user_panic("live result", || {
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
        let update = StreamUpdate {
            time_start,
            time_duration,
            classifications: classes,
        };
        unsafe { deliver(user_info, "live result", LiveEvent::Update(update)) };
    });
}

unsafe extern "C" fn error_trampoline(
    user_info: *mut c_void,
    status: i32,
    error_message: *mut c_char,
) {
    let error = unsafe { from_swift(status, error_message) };
    catch_user_panic("live error", || unsafe {
        deliver(user_info, "live error", LiveEvent::Failed(error));
    });
}

unsafe extern "C" fn complete_trampoline(user_info: *mut c_void) {
    catch_user_panic("live completion", || unsafe {
        deliver(user_info, "live completion", LiveEvent::Completed);
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
    F: FnMut(LiveEvent) + Send + 'static,
{
    let handler: LiveHandler = Mutex::new(Box::new(callback));
    let context = LiveContext::new(handler);
    let mut err_msg: *mut c_char = core::ptr::null_mut();
    let handle = unsafe {
        ffi::sa_stream_start(
            result_trampoline,
            error_trampoline,
            complete_trampoline,
            context.as_ptr(),
            Some(LiveContext::RETAIN),
            Some(LiveContext::RELEASE),
            &raw mut err_msg,
        )
    };
    if handle.is_null() {
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
        _context: context,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::{
        complete_trampoline, error_trampoline, result_trampoline, LiveContext, LiveEvent,
        LiveHandler,
    };
    use crate::error::SAError;
    use crate::ffi;

    fn recording_context() -> (LiveContext, Arc<Mutex<Vec<LiveEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        let handler: LiveHandler = Mutex::new(Box::new(move |event| {
            sink.lock().expect("events").push(event);
        }));
        (LiveContext::new(handler), events)
    }

    #[test]
    fn trampolines_report_updates_failures_and_completion() {
        let (context, events) = recording_context();
        let identifier = c"speech";
        let mut rows = [ffi::ClassificationRaw {
            identifier: identifier.as_ptr().cast_mut(),
            confidence: 0.75,
        }];
        unsafe {
            result_trampoline(
                context.as_ptr(),
                1.0,
                0.5,
                rows.as_mut_ptr().cast(),
                rows.len(),
            );
            error_trampoline(
                context.as_ptr(),
                ffi::status::ANALYSIS_FAILED,
                core::ptr::null_mut(),
            );
            complete_trampoline(context.as_ptr());
        }

        let events = events.lock().expect("events");
        assert_eq!(events.len(), 3);
        match &events[0] {
            LiveEvent::Update(update) => {
                assert!((update.time_start - 1.0).abs() < f64::EPSILON);
                assert!((update.time_duration - 0.5).abs() < f64::EPSILON);
                assert_eq!(update.classifications.len(), 1);
                assert_eq!(update.classifications[0].identifier, "speech");
            }
            other => panic!("expected an update, got {other:?}"),
        }
        assert_eq!(
            events[1],
            LiveEvent::Failed(SAError::AnalysisFailed(String::new()))
        );
        assert_eq!(events[2], LiveEvent::Completed);
    }

    #[test]
    fn a_deactivated_context_ignores_late_callbacks() {
        let (context, events) = recording_context();
        let foreign = context.retained_ptr();
        drop(context);

        unsafe {
            complete_trampoline(foreign);
            result_trampoline(foreign, 0.0, 0.0, core::ptr::null_mut(), 0);
            (LiveContext::RELEASE)(foreign);
        }
        assert!(events.lock().expect("events").is_empty());
        assert_eq!(Arc::strong_count(&events), 1);
    }

    #[test]
    fn a_panicking_handler_is_contained() {
        let handler: LiveHandler = Mutex::new(Box::new(|event| {
            assert_ne!(event, LiveEvent::Completed, "handler panics on completion");
        }));
        let context = LiveContext::new(handler);
        unsafe {
            complete_trampoline(context.as_ptr());
            complete_trampoline(context.as_ptr());
        }
        assert!(context.is_active());
    }
}

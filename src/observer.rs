//! Rust equivalent of Apple's `SNResultsObserving` protocol.

use core::ffi::{c_char, c_void};
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::classifier::{Classification, ClassificationResult};
use crate::error::{from_swift, SAError};
use crate::ffi;
use crate::request::ClassifySoundRequest;

mod private {
    pub trait Sealed {}
}

/// Marker trait for Rust types that model Apple's `SNResult` protocol.
pub trait SNResult: private::Sealed {}

/// Backwards-compatible alias for [`SNResult`].
pub trait AnalysisResult: SNResult {}

impl private::Sealed for ClassificationResult {}
impl SNResult for ClassificationResult {}
impl AnalysisResult for ClassificationResult {}

/// Rust equivalent of `SNResultsObserving`.
pub trait ResultsObserver: Send {
    fn did_produce_result(&mut self, request: &ClassifySoundRequest, result: ClassificationResult);

    fn did_fail_with_error(&mut self, request: &ClassifySoundRequest, error: SAError) {
        let _ = (request, error);
    }

    fn did_complete(&mut self, request: &ClassifySoundRequest) {
        let _ = request;
    }
}

type ResultHandler = Box<dyn FnMut(&ClassifySoundRequest, ClassificationResult) + Send + 'static>;
type ErrorHandler = Box<dyn FnMut(&ClassifySoundRequest, SAError) + Send + 'static>;
type CompleteHandler = Box<dyn FnMut(&ClassifySoundRequest) + Send + 'static>;

/// Closure-based helper for [`ResultsObserver`].
#[allow(clippy::type_complexity)]
pub struct ResultsObserverFns {
    result: ResultHandler,
    error: Option<ErrorHandler>,
    completion: Option<CompleteHandler>,
}

impl ResultsObserverFns {
    pub fn new(
        on_result: impl FnMut(&ClassifySoundRequest, ClassificationResult) + Send + 'static,
    ) -> Self {
        Self {
            result: Box::new(on_result),
            error: None,
            completion: None,
        }
    }

    #[must_use]
    pub fn on_error(
        mut self,
        on_error: impl FnMut(&ClassifySoundRequest, SAError) + Send + 'static,
    ) -> Self {
        self.error = Some(Box::new(on_error));
        self
    }

    #[must_use]
    pub fn on_complete(
        mut self,
        on_complete: impl FnMut(&ClassifySoundRequest) + Send + 'static,
    ) -> Self {
        self.completion = Some(Box::new(on_complete));
        self
    }
}

impl ResultsObserver for ResultsObserverFns {
    fn did_produce_result(&mut self, request: &ClassifySoundRequest, result: ClassificationResult) {
        (self.result)(request, result);
    }

    fn did_fail_with_error(&mut self, request: &ClassifySoundRequest, error: SAError) {
        if let Some(on_error) = &mut self.error {
            on_error(request, error);
        }
    }

    fn did_complete(&mut self, request: &ClassifySoundRequest) {
        if let Some(completion) = &mut self.completion {
            completion(request);
        }
    }
}

pub(crate) struct ObserverState {
    request: ClassifySoundRequest,
    observer: Box<dyn ResultsObserver>,
}

pub(crate) type ObserverMap = BTreeMap<usize, *mut ObserverState>;

pub(crate) fn request_key(request: &ClassifySoundRequest) -> usize {
    request.as_raw() as usize
}

pub(crate) fn box_observer<O>(request: &ClassifySoundRequest, observer: O) -> *mut c_void
where
    O: ResultsObserver + 'static,
{
    Box::into_raw(Box::new(ObserverState {
        request: request.clone(),
        observer: Box::new(observer),
    }))
    .cast()
}

pub(crate) unsafe fn drop_observer(ptr: *mut c_void) {
    if !ptr.is_null() {
        // SAFETY: ptr comes from box_observer which creates a valid ObserverState box,
        // leaks it via Box::into_raw, and hands the raw pointer to Swift. Here we
        // reconstitute the box to drop it, which is safe provided drop_observer
        // is called exactly once per box_observer call (guaranteed by Drop impl).
        drop(Box::from_raw(ptr.cast::<ObserverState>()));
    }
}

pub(crate) fn clear_observers(observers: &mut ObserverMap) {
    let owned: Vec<*mut ObserverState> = observers.values().copied().collect();
    observers.clear();
    for ptr in owned {
        unsafe { drop_observer(ptr.cast()) };
    }
}

pub(crate) unsafe extern "C" fn observer_result_trampoline(
    user_info: *mut c_void,
    time_start: f64,
    time_duration: f64,
    classifications: *mut c_void,
    classification_count: usize,
) {
    if user_info.is_null() {
        return;
    }

    let _ = catch_unwind(AssertUnwindSafe(|| {
        let state = unsafe { &mut *user_info.cast::<ObserverState>() };
        let mut rows = Vec::with_capacity(classification_count);
        let typed = classifications.cast::<ffi::ClassificationRaw>();
        if !typed.is_null() {
            for idx in 0..classification_count {
                let raw = unsafe { &*typed.add(idx) };
                let identifier = if raw.identifier.is_null() {
                    String::new()
                } else {
                    unsafe { core::ffi::CStr::from_ptr(raw.identifier) }
                        .to_string_lossy()
                        .into_owned()
                };
                rows.push(Classification {
                    identifier,
                    confidence: raw.confidence,
                });
            }
        }

        state.observer.did_produce_result(
            &state.request,
            ClassificationResult {
                time_start,
                time_duration,
                classifications: rows,
            },
        );
    }));
}

pub(crate) unsafe extern "C" fn observer_error_trampoline(
    user_info: *mut c_void,
    status: i32,
    error_message: *mut c_char,
) {
    if user_info.is_null() {
        if !error_message.is_null() {
            unsafe { ffi::sa_string_free(error_message) };
        }
        return;
    }

    let _ = catch_unwind(AssertUnwindSafe(|| {
        let state = unsafe { &mut *user_info.cast::<ObserverState>() };
        let error = unsafe { from_swift(status, error_message) };
        state.observer.did_fail_with_error(&state.request, error);
    }));
}

pub(crate) unsafe extern "C" fn observer_complete_trampoline(user_info: *mut c_void) {
    if user_info.is_null() {
        return;
    }

    let _ = catch_unwind(AssertUnwindSafe(|| {
        let state = unsafe { &mut *user_info.cast::<ObserverState>() };
        state.observer.did_complete(&state.request);
    }));
}

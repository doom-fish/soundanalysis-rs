//! `SNAudioFileAnalyzer` wrapper.

use core::ffi::c_void;
use core::ptr;
use core::ptr::NonNull;
use std::ffi::CString;
use std::path::Path;

use crate::error::{from_swift, SAError};
use crate::ffi;
use crate::observer::{
    box_observer, clear_observers, drop_observer, observer_complete_trampoline,
    observer_error_trampoline, observer_result_trampoline, request_key, ObserverMap,
    ResultsObserver,
};
use crate::request::ClassifySoundRequest;

/// Safe wrapper around `SNAudioFileAnalyzer`.
#[derive(Debug)]
pub struct AudioFileAnalyzer {
    ptr: NonNull<c_void>,
    observers: ObserverMap,
}

impl AudioFileAnalyzer {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, SAError> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| SAError::InvalidArgument("non-UTF-8 audio path".into()))?;
        let path = CString::new(path)
            .map_err(|e| SAError::InvalidArgument(format!("audio path NUL byte: {e}")))?;

        let mut analyzer = ptr::null_mut();
        let mut err = ptr::null_mut();
        let status = unsafe {
            ffi::sa_audio_file_analyzer_create(path.as_ptr(), &mut analyzer, &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }

        Ok(Self {
            ptr: NonNull::new(analyzer).expect("sa_audio_file_analyzer_create returned null"),
            observers: ObserverMap::default(),
        })
    }

    pub fn add_request<O>(
        &mut self,
        request: &ClassifySoundRequest,
        observer: O,
    ) -> Result<(), SAError>
    where
        O: ResultsObserver + 'static,
    {
        let key = request_key(request);
        if self.observers.contains_key(&key) {
            return Err(SAError::InvalidArgument(
                "request already added to analyzer".into(),
            ));
        }

        let observer_ptr = box_observer(request, observer);
        let mut err = ptr::null_mut();
        let status = unsafe {
            ffi::sa_audio_file_analyzer_add_request(
                self.ptr.as_ptr(),
                request.as_raw(),
                observer_ptr,
                observer_result_trampoline,
                observer_error_trampoline,
                observer_complete_trampoline,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            unsafe { drop_observer(observer_ptr) };
            return Err(unsafe { from_swift(status, err) });
        }

        self.observers.insert(key, observer_ptr.cast());
        Ok(())
    }

    pub fn remove_request(&mut self, request: &ClassifySoundRequest) {
        unsafe { ffi::sa_audio_file_analyzer_remove_request(self.ptr.as_ptr(), request.as_raw()) };
        if let Some(observer) = self.observers.remove(&request_key(request)) {
            unsafe { drop_observer(observer.cast()) };
        }
    }

    pub fn remove_all_requests(&mut self) {
        unsafe { ffi::sa_audio_file_analyzer_remove_all_requests(self.ptr.as_ptr()) };
        clear_observers(&mut self.observers);
    }

    pub fn analyze(&mut self) -> Result<(), SAError> {
        let mut err = ptr::null_mut();
        let status = unsafe { ffi::sa_audio_file_analyzer_analyze(self.ptr.as_ptr(), &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Wraps Apple's `analyzeWithCompletionHandler:` and returns the
    /// `didReachEndOfFile` flag.
    pub fn analyze_with_completion_handler(&mut self) -> Result<bool, SAError> {
        let mut did_reach_end = false;
        let mut err = ptr::null_mut();
        let status = unsafe {
            ffi::sa_audio_file_analyzer_analyze_with_completion(
                self.ptr.as_ptr(),
                &mut did_reach_end,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(did_reach_end)
    }

    pub fn cancel_analysis(&mut self) {
        unsafe { ffi::sa_audio_file_analyzer_cancel_analysis(self.ptr.as_ptr()) };
    }
}

impl Drop for AudioFileAnalyzer {
    fn drop(&mut self) {
        if !self.observers.is_empty() {
            unsafe { ffi::sa_audio_file_analyzer_remove_all_requests(self.ptr.as_ptr()) };
            clear_observers(&mut self.observers);
        }
        unsafe { ffi::sa_audio_file_analyzer_release(self.ptr.as_ptr()) };
    }
}

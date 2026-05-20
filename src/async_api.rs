//! Async API for `SoundAnalysis`
//!
//! This module provides async versions of operations when the `async` feature is enabled.
//! The async API is **executor-agnostic** and works with any async runtime (Tokio, async-std, smol, etc.).
//!
//! ## Available Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`AsyncAudioFileAnalyzer`] | Async file analysis with completion callback |
//! | [`AsyncAudioStreamAnalyzer`] | Async stream-analyzer wrapper with event streams |
//!
//! ## Runtime Agnostic Design
//!
//! This async API uses only `std` types and works with **any** async runtime:
//! - Uses callback-based Swift FFI for true async operations
//! - Uses `std::sync::{Arc, Mutex}` for synchronization
//! - Uses `std::task::{Poll, Waker}` for async primitives
//! - Uses `std::future::Future` trait
//!
//! ## Examples
//!
//! ### Basic Async File Analysis
//!
//! ```rust,ignore
//! use soundanalysis::async_api::AsyncAudioFileAnalyzer;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let analyzer = AsyncAudioFileAnalyzer::new("path/to/audio.mp3")?;
//!     analyzer.analyze().await?;
//!     println!("Analysis complete");
//!     Ok(())
//! }
//! ```
//!
//! ## Async vs Blocking API
//!
//! The `async_api` module provides `Future`-based wrappers around the blocking
//! `SoundAnalysis` APIs. Use this when:
//! - You have an async runtime and want to avoid blocking threads
//! - You want to run multiple analyses concurrently
//! - You're already in an async context
//!
//! For synchronous/blocking use, use the standard `AudioFileAnalyzer` API.
//!
//! ## Note on Delegates
//!
//! `SNAudioStreamAnalyzer` is exposed here through a bounded async event stream.
//! Live microphone helpers in [`crate::live`] remain available for direct callback-based use.

use crate::classifier::ClassificationResult;
use crate::error::SAError;
use crate::observer::ResultsObserverFns;
use crate::request::ClassifySoundRequest;
use crate::streaming::{AudioStreamAnalyzer, AudioStreamFormat, PcmBuffer};
use doom_fish_utils::completion::{error_from_cstr, AsyncCompletion, AsyncCompletionFuture};
use doom_fish_utils::stream::{BoundedAsyncStream, NextItem};
use std::ffi::c_void;
use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::pin::Pin;
use std::task::{Context, Poll};

// ============================================================================
// AsyncAudioFileAnalyzer - True async with callback-based FFI
// ============================================================================

/// Callback from Swift FFI for file analyzer completion
extern "C" fn analyzer_complete_callback(
    success: bool,
    error: *const i8,
    user_data: *mut c_void,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if success {
            unsafe { AsyncCompletion::<()>::complete_ok(user_data, ()) };
        } else {
            let error_msg = unsafe { error_from_cstr(error) };
            unsafe { AsyncCompletion::<()>::complete_err(user_data, error_msg) };
        }
    }));
}

/// Future type for async file analysis
pub struct AnalyzeFileFuture {
    inner: AsyncCompletionFuture<()>,
}

impl Future for AnalyzeFileFuture {
    type Output = Result<(), SAError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(SAError::AnalysisFailed))
    }
}

/// Async wrapper around `SNAudioFileAnalyzer`
///
/// This type provides async versions of the file analysis operations.
/// Unlike the blocking `AudioFileAnalyzer`, this doesn't retain the analyzer
/// across calls — each `analyze()` awaits completion and then drops internally.
#[derive(Debug)]
pub struct AsyncAudioFileAnalyzer {
    /// Path to the audio file
    path: std::ffi::CString,
}

/// One event produced by [`AsyncAudioStreamAnalyzer`].
#[derive(Debug, Clone, PartialEq)]
pub enum AudioStreamAnalysisEvent {
    Result(ClassificationResult),
    Error(SAError),
    Complete,
}

/// Async stream of [`AudioStreamAnalysisEvent`] values.
#[derive(Debug)]
pub struct AudioStreamAnalysisStream {
    inner: BoundedAsyncStream<AudioStreamAnalysisEvent>,
}

impl AudioStreamAnalysisStream {
    #[must_use]
    pub const fn next(&self) -> NextItem<'_, AudioStreamAnalysisEvent> {
        self.inner.next()
    }

    #[must_use]
    pub fn try_next(&self) -> Option<AudioStreamAnalysisEvent> {
        self.inner.try_next()
    }

    #[must_use]
    pub fn buffered_count(&self) -> usize {
        self.inner.buffered_count()
    }

    #[must_use]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

/// Async wrapper around [`AudioStreamAnalyzer`].
#[derive(Debug)]
pub struct AsyncAudioStreamAnalyzer {
    inner: AudioStreamAnalyzer,
}

impl AsyncAudioFileAnalyzer {
    /// Create a new async file analyzer for the given audio path
    ///
    /// # Errors
    ///
    /// Returns an error if the path contains invalid UTF-8 or NUL bytes.
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, SAError> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| SAError::InvalidArgument("non-UTF-8 audio path".into()))?;
        let path = std::ffi::CString::new(path)
            .map_err(|e| SAError::InvalidArgument(format!("audio path NUL byte: {e}")))?;

        Ok(Self { path })
    }

    /// Asynchronously analyze the audio file with the added requests.
    ///
    /// This method must be called after `add_request()` to process the audio
    /// and invoke the observer callbacks. The future completes when analysis
    /// is done.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying FFI call fails or the Swift callback
    /// signals an error.
    pub fn analyze(&self) -> AnalyzeFileFuture {
        let (future, ctx) = AsyncCompletion::create();
        // SAFETY: analyzer_complete_callback receives a user_data pointer that is
        // the AsyncCompletion context created above. The callback is called by the
        // Swift bridge when analysis completes, and ctx is valid for the lifetime of
        // the returned AnalyzeFileFuture (held in the inner AsyncCompletionFuture).
        // The callback is wrapped with catch_unwind to prevent panics across FFI.
        unsafe {
            crate::ffi::sa_audio_file_analyzer_analyze_async(
                self.path.as_ptr(),
                analyzer_complete_callback,
                ctx,
            );
        }
        AnalyzeFileFuture { inner: future }
    }
}

impl AsyncAudioStreamAnalyzer {
    /// Create a new async stream analyzer for the supplied audio format.
    ///
    /// # Errors
    ///
    /// Returns an error if `SoundAnalysis` cannot create the analyzer.
    pub fn new(format: AudioStreamFormat) -> Result<Self, SAError> {
        Ok(Self {
            inner: AudioStreamAnalyzer::new(format)?,
        })
    }

    #[must_use]
    pub fn format(&self) -> AudioStreamFormat {
        self.inner.format()
    }

    /// Attach a request and receive its observer callbacks as an async stream.
    ///
    /// # Errors
    ///
    /// Returns an error if the request cannot be attached to the analyzer.
    pub fn add_request_stream(
        &mut self,
        request: &ClassifySoundRequest,
        capacity: usize,
    ) -> Result<AudioStreamAnalysisStream, SAError> {
        let (stream, sender) = BoundedAsyncStream::new(capacity);
        let result_sender = sender.clone();
        let error_sender = sender.clone();
        let complete_sender = sender;
        self.inner.add_request(
            request,
            ResultsObserverFns::new(move |_, result| {
                result_sender.push(AudioStreamAnalysisEvent::Result(result));
            })
            .on_error(move |_, error| {
                error_sender.push(AudioStreamAnalysisEvent::Error(error));
            })
            .on_complete(move |_| {
                complete_sender.push(AudioStreamAnalysisEvent::Complete);
            }),
        )?;
        Ok(AudioStreamAnalysisStream { inner: stream })
    }

    pub fn remove_request(&mut self, request: &ClassifySoundRequest) {
        self.inner.remove_request(request);
    }

    pub fn remove_all_requests(&mut self) {
        self.inner.remove_all_requests();
    }

    /// Feed a PCM buffer into the analyzer.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer shape or format does not match the analyzer.
    pub fn analyze_audio_buffer(
        &mut self,
        buffer: PcmBuffer<'_>,
        audio_frame_position: i64,
    ) -> Result<(), SAError> {
        self.inner.analyze_audio_buffer(buffer, audio_frame_position)
    }

    pub fn complete_analysis(&mut self) {
        self.inner.complete_analysis();
    }
}

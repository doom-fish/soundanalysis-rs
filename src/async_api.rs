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
//! ```rust,no_run
//! use soundanalysis::async_api::{AsyncAudioFileAnalyzer, AudioStreamAnalysisEvent};
//! use soundanalysis::ClassifySoundRequest;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let mut analyzer = AsyncAudioFileAnalyzer::new("path/to/audio.mp3")?;
//! let request = ClassifySoundRequest::version1()?;
//! let events = analyzer.add_request_stream(&request, 64)?;
//! let reached_end = analyzer.analyze().await?;
//! while let Some(event) = events.try_next() {
//!     if let AudioStreamAnalysisEvent::Result(result) = event {
//!         println!("{:?}", result.top());
//!     }
//! }
//! println!("analysis complete, reached end: {reached_end}");
//! # Ok(())
//! # }
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
use crate::error::{from_swift, SAError};
use crate::file::AudioFileAnalyzer;
use crate::observer::ResultsObserverFns;
use crate::request::ClassifySoundRequest;
use crate::streaming::{AudioStreamAnalyzer, AudioStreamFormat, PcmBuffer};
use core::ffi::c_char;
use core::ops::{Deref, DerefMut};
use doom_fish_utils::completion::{AsyncCompletion, AsyncCompletionFuture};
use doom_fish_utils::panic_safe::catch_user_panic;
use doom_fish_utils::stream::{BoundedAsyncStream, NextItem};
use std::ffi::c_void;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

type AnalysisOutcome = Result<bool, SAError>;

// ============================================================================
// AsyncAudioFileAnalyzer - True async with callback-based FFI
// ============================================================================

/// Callback from Swift FFI for file analyzer completion
unsafe extern "C" fn analyzer_complete_callback(
    context: *mut c_void,
    status: i32,
    reached_end_of_file: bool,
    error_message: *mut c_char,
) {
    let outcome = if status == crate::ffi::status::OK {
        Ok(reached_end_of_file)
    } else {
        Err(unsafe { from_swift(status, error_message) })
    };
    catch_user_panic("analyzer_complete_callback", || unsafe {
        AsyncCompletion::<AnalysisOutcome>::complete_ok(context, outcome);
    });
}

/// Future type for async file analysis
pub struct AnalyzeFileFuture<'a> {
    analyzer: &'a mut AsyncAudioFileAnalyzer,
    completion: AsyncCompletionFuture<AnalysisOutcome>,
    running: bool,
}

impl Future for AnalyzeFileFuture<'_> {
    type Output = Result<bool, SAError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        let outcome = std::task::ready!(Pin::new(&mut this.completion).poll(cx));
        this.running = false;
        Poll::Ready(
            outcome
                .map_err(SAError::AnalysisFailed)
                .and_then(|result| result),
        )
    }
}

impl Drop for AnalyzeFileFuture<'_> {
    fn drop(&mut self) {
        if self.running {
            self.analyzer.inner.cancel_analysis();
        }
    }
}

/// Async wrapper around `SNAudioFileAnalyzer`
///
/// Dereferences to [`AudioFileAnalyzer`] for request management and adds an
/// async [`analyze`](Self::analyze) plus [`add_request_stream`](Self::add_request_stream).
#[derive(Debug)]
pub struct AsyncAudioFileAnalyzer {
    inner: AudioFileAnalyzer,
}

impl Deref for AsyncAudioFileAnalyzer {
    type Target = AudioFileAnalyzer;

    fn deref(&self) -> &AudioFileAnalyzer {
        &self.inner
    }
}

impl DerefMut for AsyncAudioFileAnalyzer {
    fn deref_mut(&mut self) -> &mut AudioFileAnalyzer {
        &mut self.inner
    }
}

fn analysis_stream(
    capacity: usize,
) -> Result<(AudioStreamAnalysisStream, ResultsObserverFns), SAError> {
    if capacity == 0 {
        return Err(SAError::InvalidArgument(
            "event stream capacity must be greater than zero".into(),
        ));
    }
    let (stream, sender) = BoundedAsyncStream::new(capacity);
    let result_sender = sender.clone();
    let error_sender = sender.clone();
    let complete_sender = sender;
    let observer = ResultsObserverFns::new(move |_, result| {
        result_sender.push(AudioStreamAnalysisEvent::Result(result));
    })
    .on_error(move |_, error| {
        error_sender.push(AudioStreamAnalysisEvent::Error(error));
    })
    .on_complete(move |_| {
        complete_sender.push(AudioStreamAnalysisEvent::Complete);
    });
    Ok((AudioStreamAnalysisStream { inner: stream }, observer))
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
    /// Returns an error if the path contains invalid UTF-8 or NUL bytes, or if
    /// `SoundAnalysis` cannot open the audio file.
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, SAError> {
        Ok(Self {
            inner: AudioFileAnalyzer::new(path)?,
        })
    }

    pub fn add_request_stream(
        &mut self,
        request: &ClassifySoundRequest,
        capacity: usize,
    ) -> Result<AudioStreamAnalysisStream, SAError> {
        let (stream, observer) = analysis_stream(capacity)?;
        self.inner.add_request(request, observer)?;
        Ok(stream)
    }

    /// Asynchronously analyze the audio file with the added requests.
    ///
    /// Add requests first with [`add_request_stream`](Self::add_request_stream)
    /// or [`AudioFileAnalyzer::add_request`]; their observers receive the
    /// results while the analysis runs. The future resolves to whether the
    /// analysis reached the end of the file, which is `false` after
    /// [`AudioFileAnalyzer::cancel_analysis`].
    ///
    /// # Errors
    ///
    /// Returns an error if another analysis is still running on this analyzer
    /// or if an observer reports an analysis failure.
    pub fn analyze(&mut self) -> AnalyzeFileFuture<'_> {
        let (completion, context) = AsyncCompletion::<AnalysisOutcome>::create();
        let mut err = core::ptr::null_mut();
        let status = unsafe {
            crate::ffi::sa_audio_file_analyzer_analyze_async(
                self.inner.ptr.as_ptr(),
                analyzer_complete_callback,
                context,
                &raw mut err,
            )
        };
        let running = status == crate::ffi::status::OK;
        if !running {
            let error = unsafe { from_swift(status, err) };
            unsafe { AsyncCompletion::<AnalysisOutcome>::complete_ok(context, Err(error)) };
        }
        AnalyzeFileFuture {
            analyzer: self,
            completion,
            running,
        }
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
        let (stream, observer) = analysis_stream(capacity)?;
        self.inner.add_request(request, observer)?;
        Ok(stream)
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
        self.inner
            .analyze_audio_buffer(buffer, audio_frame_position)
    }

    pub fn complete_analysis(&mut self) {
        self.inner.complete_analysis();
    }
}

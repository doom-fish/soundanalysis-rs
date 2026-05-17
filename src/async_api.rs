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
//! `SNAudioStreamAnalyzer` uses a continuous delegate pattern (fires multiple times).
//! This is deferred to Tier 2 for a Stream-based API. For now, use the blocking
//! `AudioStreamAnalyzer` if you need streaming analysis.

use crate::error::SAError;
use doom_fish_utils::completion::{error_from_cstr, AsyncCompletion, AsyncCompletionFuture};
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

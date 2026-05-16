//! File-based sound classification — wraps `SNAudioFileAnalyzer` +
//! `SNClassifySoundRequest` with Apple's built-in `version1` classifier.

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;
use std::path::Path;

use crate::error::{from_swift, SAError};
use crate::ffi;

/// One ranked classification at one point in time.
#[derive(Debug, Clone, PartialEq)]
pub struct Classification {
    /// Apple's category identifier (e.g. `"speech"`, `"music"`,
    /// `"applause"`, `"dog_bark"`). See [`known_classifications`] for the
    /// full set returned by `version1`.
    pub identifier: String,
    /// Confidence in `0.0..=1.0`. Higher is more confident.
    pub confidence: f64,
}

/// One analysis-window's classifications.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassificationResult {
    /// Window start (seconds since file start).
    pub time_start: f64,
    /// Window duration (seconds).
    pub time_duration: f64,
    /// All classifications returned for this window, ordered as Apple
    /// returns them (typically descending confidence within the top-N).
    pub classifications: Vec<Classification>,
}

impl ClassificationResult {
    /// Convenience: the highest-confidence classification in this window.
    #[must_use]
    pub fn top(&self) -> Option<&Classification> {
        self.classifications
            .iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(core::cmp::Ordering::Equal))
    }
}

/// Synchronously classify the audio file at `path` using Apple's built-in
/// `version1` classifier (~300 everyday-sound categories).
///
/// Supports any audio format `AVFoundation` can read (AIFF, WAV, M4A, MP3, …).
///
/// # Errors
///
/// * [`SAError::InvalidArgument`] — `path` is not valid UTF-8 or contains
///   an interior NUL byte.
/// * [`SAError::AudioLoadFailed`] — `AVFoundation` can't decode the file.
/// * [`SAError::RequestCreateFailed`] — the built-in classifier can't be
///   loaded (very unlikely on a working install).
/// * [`SAError::AnalysisFailed`] — the analyzer reported an error mid-run.
///
/// # Examples
///
/// ```rust,no_run
/// use soundanalysis::classify_file;
///
/// let results = classify_file("/tmp/cough.wav").unwrap();
/// for r in &results {
///     if let Some(top) = r.top() {
///         println!("{:.2}s: {} ({:.2})", r.time_start, top.identifier, top.confidence);
///     }
/// }
/// ```
pub fn classify_file(path: impl AsRef<Path>) -> Result<Vec<ClassificationResult>, SAError> {
    let path_str = path
        .as_ref()
        .to_str()
        .ok_or_else(|| SAError::InvalidArgument("non-UTF-8 path".into()))?;
    let path_c = CString::new(path_str)
        .map_err(|e| SAError::InvalidArgument(format!("path NUL byte: {e}")))?;

    let mut array: *mut c_void = ptr::null_mut();
    let mut count: usize = 0;
    let mut err_msg: *mut c_char = ptr::null_mut();

    let status = unsafe {
        ffi::sa_classify_file(path_c.as_ptr(), &mut array, &mut count, &mut err_msg)
    };
    if status != ffi::status::OK {
        return Err(unsafe { from_swift(status, err_msg) });
    }
    if array.is_null() || count == 0 {
        return Ok(Vec::new());
    }

    let typed = array.cast::<ffi::ClassificationResultRaw>();
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let raw = unsafe { &*typed.add(i) };
        let mut classifications = Vec::with_capacity(raw.classification_count);
        for j in 0..raw.classification_count {
            let craw = unsafe { &*raw.classifications.add(j) };
            let identifier = if craw.identifier.is_null() {
                String::new()
            } else {
                unsafe { core::ffi::CStr::from_ptr(craw.identifier) }
                    .to_string_lossy()
                    .into_owned()
            };
            classifications.push(Classification {
                identifier,
                confidence: craw.confidence,
            });
        }
        out.push(ClassificationResult {
            time_start: raw.time_start,
            time_duration: raw.time_duration,
            classifications,
        });
    }
    unsafe { ffi::sa_classification_results_free(array, count) };
    Ok(out)
}

/// Classify the audio at `audio_path` against a custom Core ML
/// model file (`.mlmodel` or `.mlpackage`). Useful for shipping a
/// domain-specific sound classifier trained with Create ML.
///
/// # Errors
///
/// Returns [`SAError`] if either file can't be loaded or
/// `SNAudioFileAnalyzer` fails.
pub fn classify_file_with_model(
    audio_path: impl AsRef<Path>,
    model_path: impl AsRef<Path>,
) -> Result<Vec<ClassificationResult>, SAError> {
    let a = audio_path
        .as_ref()
        .to_str()
        .ok_or_else(|| SAError::InvalidArgument("non-UTF-8 audio path".into()))?;
    let m = model_path
        .as_ref()
        .to_str()
        .ok_or_else(|| SAError::InvalidArgument("non-UTF-8 model path".into()))?;
    let a_c = CString::new(a)
        .map_err(|e| SAError::InvalidArgument(format!("audio path NUL: {e}")))?;
    let m_c = CString::new(m)
        .map_err(|e| SAError::InvalidArgument(format!("model path NUL: {e}")))?;

    let mut array: *mut c_void = ptr::null_mut();
    let mut count: usize = 0;
    let mut err_msg: *mut c_char = ptr::null_mut();
    let status = unsafe {
        ffi::sa_classify_file_with_model(
            a_c.as_ptr(),
            m_c.as_ptr(),
            &mut array,
            &mut count,
            &mut err_msg,
        )
    };
    if status != ffi::status::OK {
        return Err(unsafe { from_swift(status, err_msg) });
    }
    if array.is_null() || count == 0 {
        return Ok(Vec::new());
    }
    let typed = array.cast::<ffi::ClassificationResultRaw>();
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let raw = unsafe { &*typed.add(i) };
        let mut classifications = Vec::with_capacity(raw.classification_count);
        for j in 0..raw.classification_count {
            let craw = unsafe { &*raw.classifications.add(j) };
            let identifier = if craw.identifier.is_null() {
                String::new()
            } else {
                unsafe { core::ffi::CStr::from_ptr(craw.identifier) }
                    .to_string_lossy()
                    .into_owned()
            };
            classifications.push(Classification {
                identifier,
                confidence: craw.confidence,
            });
        }
        out.push(ClassificationResult {
            time_start: raw.time_start,
            time_duration: raw.time_duration,
            classifications,
        });
    }
    unsafe { ffi::sa_classification_results_free(array, count) };
    Ok(out)
}

/// All sound categories that Apple's built-in `version1` classifier can
/// recognise (typically 300+ items: `"speech"`, `"music"`, `"applause"`,
/// `"dog_bark"`, `"engine_idling"`, `"wind"`, …).
///
/// # Errors
///
/// Returns [`SAError::RequestCreateFailed`] if the built-in classifier
/// can't be initialised.
pub fn known_classifications() -> Result<Vec<String>, SAError> {
    let mut array: *mut *mut c_char = ptr::null_mut();
    let mut count: usize = 0;
    let status = unsafe { ffi::sa_known_classifications(&mut array, &mut count) };
    if status != ffi::status::OK {
        return Err(SAError::RequestCreateFailed(
            "built-in classifier unavailable".into(),
        ));
    }
    if array.is_null() || count == 0 {
        return Ok(Vec::new());
    }
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let p = unsafe { *array.add(i) };
        if p.is_null() {
            continue;
        }
        let s = unsafe { core::ffi::CStr::from_ptr(p) }
            .to_string_lossy()
            .into_owned();
        out.push(s);
    }
    unsafe { ffi::sa_known_classifications_free(array, count) };
    Ok(out)
}

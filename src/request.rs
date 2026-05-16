//! `SNClassifySoundRequest` and request-related helpers.

use core::ffi::c_void;
use core::ptr;
use core::ptr::NonNull;
use std::ffi::CString;
use std::path::Path;

use crate::error::{from_swift, SAError};
use crate::ffi;
use crate::time::{decode_constraint_raw, TimeDurationConstraint};

mod private {
    pub trait Sealed {}
}

/// Marker trait for Rust types that model Apple's `SNRequest` protocol.
pub trait AnalysisRequest: private::Sealed {}

/// Apple's built-in sound classifier identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassifierIdentifier {
    Version1,
}

impl ClassifierIdentifier {
    const fn as_ffi(self) -> i32 {
        match self {
            Self::Version1 => ffi::classifier_identifier::VERSION1,
        }
    }
}

/// Safe handle to `SNClassifySoundRequest`.
#[derive(Debug)]
pub struct ClassifySoundRequest {
    ptr: NonNull<c_void>,
}

impl private::Sealed for ClassifySoundRequest {}
impl AnalysisRequest for ClassifySoundRequest {}

impl Clone for ClassifySoundRequest {
    fn clone(&self) -> Self {
        let ptr = unsafe { ffi::sa_request_retain(self.ptr.as_ptr()) };
        Self {
            ptr: NonNull::new(ptr).expect("sa_request_retain returned null"),
        }
    }
}

impl Drop for ClassifySoundRequest {
    fn drop(&mut self) {
        unsafe { ffi::sa_request_release(self.ptr.as_ptr()) };
    }
}

impl ClassifySoundRequest {
    /// Create a request for Apple's built-in `version1` classifier.
    pub fn version1() -> Result<Self, SAError> {
        Self::with_classifier_identifier(ClassifierIdentifier::Version1)
    }

    /// Create a request for a known Apple-provided classifier.
    pub fn with_classifier_identifier(classifier: ClassifierIdentifier) -> Result<Self, SAError> {
        let mut request = ptr::null_mut();
        let mut err = ptr::null_mut();
        let status = unsafe {
            ffi::sa_request_create_classifier(classifier.as_ffi(), &mut request, &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(Self {
            ptr: NonNull::new(request).expect("sa_request_create_classifier returned null"),
        })
    }

    /// Create a request backed by a custom Core ML sound-classification model.
    ///
    /// The model path should point at a compiled `.mlmodelc` directory or a
    /// packaged `.mlpackage` model.
    pub fn with_model_file(path: impl AsRef<Path>) -> Result<Self, SAError> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| SAError::InvalidArgument("non-UTF-8 model path".into()))?;
        let path = CString::new(path)
            .map_err(|e| SAError::InvalidArgument(format!("model path NUL byte: {e}")))?;

        let mut request = ptr::null_mut();
        let mut err = ptr::null_mut();
        let status = unsafe {
            ffi::sa_request_create_model(path.as_ptr(), &mut request, &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(Self {
            ptr: NonNull::new(request).expect("sa_request_create_model returned null"),
        })
    }

    #[must_use]
    pub fn overlap_factor(&self) -> f64 {
        unsafe { ffi::sa_request_get_overlap_factor(self.ptr.as_ptr()) }
    }

    pub fn set_overlap_factor(&mut self, overlap_factor: f64) -> Result<(), SAError> {
        if !(0.0..1.0).contains(&overlap_factor) {
            return Err(SAError::InvalidArgument(
                "overlap_factor must be in 0.0..1.0".into(),
            ));
        }

        let mut err = ptr::null_mut();
        let status = unsafe {
            ffi::sa_request_set_overlap_factor(self.ptr.as_ptr(), overlap_factor, &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    #[must_use]
    pub fn window_duration(&self) -> f64 {
        unsafe { ffi::sa_request_get_window_duration(self.ptr.as_ptr()) }
    }

    pub fn set_window_duration(&mut self, seconds: f64) -> Result<(), SAError> {
        if !(seconds.is_finite() && seconds > 0.0) {
            return Err(SAError::InvalidArgument(
                "window duration must be finite and > 0".into(),
            ));
        }

        let mut err = ptr::null_mut();
        let status = unsafe {
            ffi::sa_request_set_window_duration(self.ptr.as_ptr(), seconds, &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn window_duration_constraint(&self) -> Result<TimeDurationConstraint, SAError> {
        let mut raw = ffi::TimeDurationConstraintRaw::empty();
        let mut err = ptr::null_mut();
        let status = unsafe {
            ffi::sa_request_get_window_duration_constraint(self.ptr.as_ptr(), &mut raw, &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(decode_constraint_raw(raw))
    }

    pub fn known_classifications(&self) -> Result<Vec<String>, SAError> {
        let mut array = ptr::null_mut();
        let mut count = 0;
        let mut err = ptr::null_mut();
        let status = unsafe {
            ffi::sa_request_known_classifications_for_request(
                self.ptr.as_ptr(),
                &mut array,
                &mut count,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        if array.is_null() || count == 0 {
            return Ok(Vec::new());
        }

        let mut out = Vec::with_capacity(count);
        for idx in 0..count {
            let value = unsafe { *array.add(idx) };
            if value.is_null() {
                continue;
            }
            out.push(
                unsafe { core::ffi::CStr::from_ptr(value) }
                    .to_string_lossy()
                    .into_owned(),
            );
        }
        unsafe { ffi::sa_known_classifications_free(array, count) };
        Ok(out)
    }

    pub(crate) fn as_raw(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

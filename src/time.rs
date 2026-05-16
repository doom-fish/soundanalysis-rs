//! Rust representation of `SNTimeDurationConstraint`.

use core::ptr;

use crate::error::{from_swift, SAError};
use crate::ffi;

/// A `CMTimeRange` represented as seconds for ergonomic Rust access.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeRange {
    pub start_seconds: f64,
    pub duration_seconds: f64,
}

/// Discriminator for [`TimeDurationConstraint`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeDurationConstraintType {
    Enumerated,
    Range,
}

/// Safe Rust representation of Apple's refined-for-Swift
/// `SNTimeDurationConstraint` enum.
#[derive(Debug, Clone, PartialEq)]
pub enum TimeDurationConstraint {
    Enumerated(Vec<f64>),
    Range(TimeRange),
}

impl TimeDurationConstraint {
    /// Construct an enumerated duration constraint.
    pub fn enumerated(durations: impl IntoIterator<Item = f64>) -> Result<Self, SAError> {
        let durations: Vec<f64> = durations.into_iter().collect();
        if durations.is_empty() {
            return Err(SAError::InvalidArgument(
                "enumerated constraints need at least one duration".into(),
            ));
        }

        let mut raw = ffi::TimeDurationConstraintRaw::empty();
        let mut err = ptr::null_mut();
        let status = unsafe {
            ffi::sa_time_duration_constraint_create_enumerated(
                durations.as_ptr(),
                durations.len(),
                &mut raw,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(decode_constraint_raw(raw))
    }

    /// Construct a range duration constraint.
    pub fn range(start_seconds: f64, duration_seconds: f64) -> Result<Self, SAError> {
        if duration_seconds < 0.0 {
            return Err(SAError::InvalidArgument(
                "duration range cannot be negative".into(),
            ));
        }

        let mut raw = ffi::TimeDurationConstraintRaw::empty();
        let mut err = ptr::null_mut();
        let status = unsafe {
            ffi::sa_time_duration_constraint_create_range(
                start_seconds,
                duration_seconds,
                &mut raw,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(decode_constraint_raw(raw))
    }

    #[must_use]
    pub fn constraint_type(&self) -> TimeDurationConstraintType {
        match self {
            Self::Enumerated(_) => TimeDurationConstraintType::Enumerated,
            Self::Range(_) => TimeDurationConstraintType::Range,
        }
    }

    #[must_use]
    pub fn enumerated_durations(&self) -> Option<&[f64]> {
        match self {
            Self::Enumerated(durations) => Some(durations),
            Self::Range(_) => None,
        }
    }

    #[must_use]
    pub fn duration_range(&self) -> Option<TimeRange> {
        match self {
            Self::Enumerated(_) => None,
            Self::Range(range) => Some(*range),
        }
    }
}

pub(crate) fn decode_constraint_raw(raw: ffi::TimeDurationConstraintRaw) -> TimeDurationConstraint {
    let constraint = match raw.kind {
        ffi::constraint_type::ENUMERATED => {
            let mut durations = Vec::with_capacity(raw.value_count);
            if !raw.values.is_null() {
                for idx in 0..raw.value_count {
                    durations.push(unsafe { *raw.values.add(idx) });
                }
            }
            TimeDurationConstraint::Enumerated(durations)
        }
        ffi::constraint_type::RANGE => TimeDurationConstraint::Range(TimeRange {
            start_seconds: raw.range_start_seconds,
            duration_seconds: raw.range_duration_seconds,
        }),
        other => panic!("unexpected SoundAnalysis constraint kind {other}"),
    };

    if !raw.values.is_null() {
        unsafe { ffi::sa_double_array_free(raw.values, raw.value_count) };
    }

    constraint
}

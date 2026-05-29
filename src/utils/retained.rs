//! Declarative macro for retain/release wrapper boilerplate.
//!
//! Several wrapper types hold a single `NonNull<c_void>` pointer to a retained
//! Objective-C / Swift object and hand-roll identical `Clone` (retain) and
//! `Drop` (release) implementations. `sa_retained!` consolidates that
//! boilerplate into a single audited place.
//!
//! The generated impls preserve the exact behavior of the previous
//! hand-written versions:
//! - `Clone` bumps the retain count by calling the supplied `retain` FFI fn and
//!   wraps the result in `NonNull`, panicking if the bridge returns null
//!   (matching the original `.expect(...)` guards).
//! - `Drop` decrements the retain count by calling the supplied `release` FFI
//!   fn exactly once.
//!
//! Types whose `Drop` carries extra logic beyond a single retain/release call
//! (e.g. `AudioFileAnalyzer` / `AudioStreamAnalyzer`, which tear down their
//! observer maps before releasing) are intentionally left hand-written.

/// Generate `Clone` and/or `Drop` impls for a `NonNull` retain/release wrapper.
///
/// Variants:
/// - Full `Clone` + `Drop`:
///   `sa_retained!(Ty, field = ptr, retain = path::retain, release = path::release);`
/// - `Drop` only:
///   `sa_retained!(Ty, field = ptr, release = path::release);`
macro_rules! sa_retained {
    // Clone + Drop
    ($ty:ty, field = $field:ident, retain = $retain:path, release = $release:path $(,)?) => {
        impl Clone for $ty {
            fn clone(&self) -> Self {
                // SAFETY: the retain FFI fn increments the retain count on the
                // underlying object and returns a valid, non-null pointer to it.
                let ptr = unsafe { $retain(self.$field.as_ptr()) };
                Self {
                    $field: core::ptr::NonNull::new(ptr)
                        .expect(concat!(stringify!($retain), " returned null")),
                }
            }
        }

        impl Drop for $ty {
            fn drop(&mut self) {
                // SAFETY: the release FFI fn decrements the retain count once,
                // matching the single retain performed in `clone`.
                unsafe { $release(self.$field.as_ptr()) };
            }
        }
    };

    // Drop only
    ($ty:ty, field = $field:ident, release = $release:path $(,)?) => {
        impl Drop for $ty {
            fn drop(&mut self) {
                // SAFETY: the release FFI fn decrements the retain count once.
                unsafe { $release(self.$field.as_ptr()) };
            }
        }
    };
}

pub(crate) use sa_retained;

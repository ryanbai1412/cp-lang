//! Moving values between places. Core re-exports `swap`, so no module imports it.
//!
//! start: `swap`
//! example: `var a = 1; var b = 2; swap(mut a, mut b); (a, b) == (2, 1)`

/// `swap(mut a, mut b)`: each place takes the other's value (R05.8.2).
pub fn swap<T>(mut a: T, mut b: T):
    (a, b) = (b, a)

//! CP Lang has one edition, 0.3 (`spec/`, with `spec/std` as its standard
//! library). The checks below are what is left of edition switching and go
//! as their call sites are simplified.

pub const fn is_v03() -> bool {
    true
}

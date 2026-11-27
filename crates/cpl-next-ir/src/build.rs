//! Facts about the build a program is checked for, passed down by the pipeline as plain data.

/// The judge a program is built for, as far as checking reads it (R03.1.9).
#[derive(Clone, Debug)]
pub struct Judge {
    pub name: String,
    pub memory_mb: u64,
}

//! What the bounds prover shows in hovers, per revision of a file: the
//! session lint keeps for it, so the diagnostics and every hover share one
//! proof of the file.

pub(super) type Proven = cpl_lint::Session;

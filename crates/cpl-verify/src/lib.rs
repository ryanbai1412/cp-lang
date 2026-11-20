//! Verification harness for the compiler and its tools.
//!
//! - [`fuzz`]: random programs with a known answer.
//! - [`backend`]: the transpilers, interpreters and C++ compilers under test.
//! - [`diff`]: the differential check and the fuzzing campaign.
//! - [`layout`]: the `sizeof` audit.
//! - [`loc`]: line counts, raw and library-stripped.
//! - [`bench`]: toolchain latency against the budgets.
//! - [`lsp`], [`json`]: the LSP client the benchmark drives.

pub mod backend;
pub mod bench;
pub mod diff;
pub mod fuzz;
pub mod json;
pub mod layout;
pub mod loc;
pub mod lsp;
pub mod process;

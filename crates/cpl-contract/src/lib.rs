//! The data contracts 0 and 9 pass between a tool, the facade and the pipeline: one
//! definition each, so the facade's API names the same types whichever pipeline answers it.

pub use cpl_diag::FileId;
use cpl_diag::Span;
pub use cpl_prim::Prim;
use std::sync::Arc;

/// A file's exact text.
pub type Text = Arc<str>;

/// Where a module the snapshot's files import comes from when the snapshot
/// does not hold it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum World {
    /// The folders `cp.toml` and `lib.toml` name, on disk. A file is read
    /// once, the first time a query needs it, and kept for the session.
    #[default]
    Disk,
    /// Nothing outside the snapshot: a host with no file system (the
    /// browser, a sandbox). `use kit.m` finds `kit/m.cp` among the files,
    /// through the `cp.toml` among them.
    Closed,
}

/// Where a name is declared: a span of the file at `path`, which may be
/// another file of the snapshot or a module of std.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Definition {
    pub path: String,
    pub span: Span,
}

/// What an editor shows for a place in a file.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Hover {
    /// The name or expression asked about, in file positions.
    pub span: Span,
    pub text: String,
    /// The `///` comment above the declaration, wherever it is declared.
    pub doc: Option<String>,
    /// The cost the doc states, in the receiver's domains:
    /// `O(log |V|) amortized`.
    pub cost: Option<String>,
    /// What kind of value it is, when it has a type.
    pub typed: Option<Typed>,
    /// The text is a fn's signature, `fn name(..) -> ..`.
    pub signature: bool,
}

/// The kinds of value an editor treats apart.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Typed {
    Int,
    Fn,
    Other,
}

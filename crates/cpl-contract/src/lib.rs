//! The data contracts 0 and 9 pass between a tool, the facade and the pipeline: one
//! definition each, so the facade's API names the same types whichever pipeline answers it.

use cpl_diag::Span;
use std::sync::Arc;

/// A file of a snapshot: its path, as the tool was given it. Stable for
/// the life of a session.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FileId(Arc<str>);

impl FileId {
    pub fn new(path: &str) -> FileId {
        FileId(Arc::from(path))
    }

    pub fn path(&self) -> &str {
        &self.0
    }
}

impl From<&str> for FileId {
    fn from(path: &str) -> FileId {
        FileId::new(path)
    }
}

impl AsRef<str> for FileId {
    fn as_ref(&self) -> &str {
        self.path()
    }
}

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

/// A core primitive type (contract 9's `Type::prim`), each by the name a program writes: the
/// one list the checker reads core type names from. Std's types are not here; a tool reaches
/// them through `Type::decl` and `Type::implements`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Prim {
    Int,
    Bool,
    Char,
    U64,
    I128,
    F64,
}

impl Prim {
    pub const ALL: [Prim; 6] = [
        Prim::Int,
        Prim::Bool,
        Prim::Char,
        Prim::U64,
        Prim::I128,
        Prim::F64,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Prim::Int => "Int",
            Prim::Bool => "Bool",
            Prim::Char => "Char",
            Prim::U64 => "U64",
            Prim::I128 => "I128",
            Prim::F64 => "F64",
        }
    }

    /// The core type a program writes as `name`.
    pub fn named(name: &str) -> Option<Prim> {
        Prim::ALL.into_iter().find(|p| p.name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::Prim;

    #[test]
    fn each_core_type_is_found_by_the_name_it_is_written() {
        for p in Prim::ALL {
            assert_eq!(Prim::named(p.name()), Some(p));
        }
        assert_eq!(Prim::named("Str"), None);
    }
}

//! What a tool still reads in today's forms: the checked file, its types
//! and hovers, the lowered program's statements, a run's recording. These
//! are answers, not entry points: nothing here checks, builds or runs a
//! program. Each use is one the rest of the facade's queries (hover,
//! complete, explain, replay: contract 9) take over in a later migration
//! stage, and debt-ratchet's `facade_today_uses` counts them until then.

use std::sync::Arc;

use cpl_diag::Diagnostic;

use crate::session::{self, Layer, Session};
use crate::snapshot::Text;

pub use cpl_check::{
    arity_key, builtin_members, closest, human_bytes, interval, is_list_form_hook, loaded_std,
    needs_order, prelude_names, prelude_type, prelude_uses, split_builtin_params, type_arity,
    Analysis, AnalyzedItem, BuiltinMember, CalleeKind, Decl, DeclKind, Facts, FsLoader, HoverEntry,
    Import, Loader, MemberKind, Module, Overflow, Param, PreludeKind, RangeEnds, Receiver,
    Representation, Stats, Symbol, Ty, TypedIndex, BUILTIN_MEMBERS, BUILTIN_TYPES, DOMAIN_MEMBERS,
    OPTIONAL_MEMBERS, POS_MEMBERS, RANGE_MEMBERS,
};

/// A run's profile.
pub mod profile {
    pub use cpl_interp::profile::*;
}

/// What the tests of the C++ count in it.
#[cfg(feature = "cpp")]
pub mod cpp_counts {
    pub use cpl_codegen::cpp_counts::*;
}

/// What `cpl check` and the editor report of a checked file:
/// [`Layer::Program`].
pub fn reported(analysis: &Analysis) -> Vec<Diagnostic> {
    session::diagnostics_of(analysis, Layer::Program)
}

macro_rules! asks {
    ($($name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;)*) => {
        $(pub fn $name(&self, path: &str $(, $arg: $ty)*) -> $ret {
            self.asked(path).db.$name(path $(, $arg)*)
        })*
    };
}

/// The checked file and the editor's questions of it, in today's forms.
impl Session {
    /// The checked form of `file`: `None` when the session has no such file.
    #[inline(always)]
    pub fn analyze(&self, file: impl AsRef<str>) -> Option<Arc<Analysis>> {
        self.checked(file.as_ref())
    }

    /// [`Session::analyze`] of the session moved on to `file` holding `text`.
    #[inline(always)]
    pub fn analyze_text(&mut self, file: impl AsRef<str>, text: impl AsRef<str>) -> Arc<Analysis> {
        *self = self.with_file(&file, text);
        self.checked(file.as_ref()).unwrap_or_default()
    }

    #[inline(always)]
    pub fn analysis(&self, file: impl AsRef<str>) -> Option<Arc<Analysis>> {
        self.analyze(file)
    }

    #[inline(always)]
    pub fn facts(&self, file: impl AsRef<str>) -> Option<Arc<Facts>> {
        self.analyze(file).map(|analysis| analysis.facts().clone())
    }

    /// The text of `path`: a file of the snapshot, a module of its
    /// standard library (`std/deque.cp`), or what an import loaded.
    pub fn file_text(&self, path: &str) -> Option<Text> {
        let snapshot = self.snapshot();
        if let Some(held) = snapshot.text_at(path) {
            return Some(held.clone());
        }
        let module = path
            .strip_prefix("std/")
            .and_then(|m| m.strip_suffix(".cp"));
        if let Some(text) = module.and_then(|m| snapshot.std_text(m)) {
            return Some(Text::from(text));
        }
        self.synced(None).db.file_text(path)
    }

    pub fn stats(&self) -> Stats {
        self.synced(None).db.stats()
    }

    pub fn import_cycles(&self) -> Vec<Vec<String>> {
        self.synced(None).db.import_cycles().to_vec()
    }

    asks! {
        method_fits(receiver: &Ty, names: &[&str]) -> Vec<Option<bool>>;
        storage() -> Option<Storage>;
        representation(offset: u32) -> Option<Representation>;
        costs_in(start: u32, end: u32) -> Vec<(cpl_diag::Span, String, String)>;
        typed_index(offset: u32, on_index: bool) -> Option<TypedIndex>;
        symbols() -> Vec<Symbol>;
        fns() -> Vec<Symbol>;
    }
}

/// The bytes a file's globals hold, and each one's when it is known.
pub type Storage = (u128, Vec<(String, Option<u64>)>);

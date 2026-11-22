//! Diagnostics: source spans, stable error codes, and rendering shared by every tool.
//!
//! Every tool (`cpl check`, `cpl run`, `cpl build`, `cpl lsp`) reports through
//! the types here, so an error reads the same on the command line and in the
//! editor (`plan/30-compiler.md` 3.7).

pub mod registry;
mod render;
mod source;

pub use registry::{lookup, CodeInfo, REGISTRY};
pub use render::{render, render_all};
pub use source::{LineCol, LineIndex};

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// A half-open byte range in one source file.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, PartialOrd, Ord)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const fn new(start: u32, end: u32) -> Span {
        Span { start, end }
    }

    /// An empty span at `at`.
    pub const fn point(at: u32) -> Span {
        Span { start: at, end: at }
    }

    pub const fn len(self) -> u32 {
        self.end - self.start
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// The smallest span covering both.
    pub fn to(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }

    /// The same span moved right by `by` bytes. Items are parsed with spans
    /// relative to their own start, so that an edit elsewhere in the file
    /// leaves them untouched; this turns them back into file positions.
    pub const fn shift(self, by: u32) -> Span {
        Span::new(self.start + by, self.end + by)
    }

    /// True when `offset` is inside the span or at its end (a cursor right
    /// after a name still hovers that name).
    pub const fn touches(self, offset: u32) -> bool {
        self.start <= offset && offset <= self.end
    }

    pub const fn contains(self, other: Span) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    pub fn text(self, src: &str) -> &str {
        &src[self.start as usize..self.end as usize]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

impl Severity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        }
    }
}

/// A stable diagnostic code such as `E0101`. Codes never change meaning, so
/// they can be looked up in the error index and matched in tests.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Code(pub &'static str);

/// Every code the front end can report. Syntax is `E00xx`/`E01xx`, names are
/// `E02xx`, types and domains are `E03xx`, ownership is `E04xx`. The back
/// ends report `E05xx` for what they can't run, lends, projections and
/// impls are `E06xx`, and a program that fails while it runs gets the `R` code of its trap (`R0001` when uncatalogued).
pub mod codes {
    use super::Code;

    pub const TAB_INDENT: Code = Code("E0001");
    pub const BAD_DEDENT: Code = Code("E0002");
    pub const UNTERMINATED_STRING: Code = Code("E0003");
    pub const UNKNOWN_CHAR: Code = Code("E0004");
    pub const BAD_NUMBER: Code = Code("E0005");
    pub const UNEXPECTED_INDENT: Code = Code("E0006");
    pub const BAD_CHAR_LITERAL: Code = Code("E0007");

    pub const UNCLOSED_DELIMITER: Code = Code("E0101");
    pub const EXPECTED_EXPR: Code = Code("E0102");
    pub const EXPECTED_TOKEN: Code = Code("E0103");
    pub const EXPECTED_BLOCK: Code = Code("E0104");
    pub const EXPECTED_TYPE: Code = Code("E0105");
    pub const EXPECTED_PATTERN: Code = Code("E0106");
    pub const UNEXPECTED_TOKEN: Code = Code("E0107");
    pub const EXPECTED_NAME: Code = Code("E0108");
    pub const UNMATCHED_CLOSER: Code = Code("E0109");
    pub const UNPARENTHESIZED_COALESCE: Code = Code("E0110");
    /// `a < b > c`, or a chain with `==`: split it with `and`.
    pub const MIXED_COMPARISON_CHAIN: Code = Code("E0111");

    pub const UNRESOLVED_NAME: Code = Code("E0201");
    pub const DUPLICATE_DEFINITION: Code = Code("E0202");
    pub const UNKNOWN_FIELD: Code = Code("E0203");
    pub const UNKNOWN_TYPE: Code = Code("E0204");
    pub const USED_BEFORE_SET: Code = Code("E0205");
    pub const UNRESOLVED_MODULE: Code = Code("E0206");
    pub const PRIVATE_NAME: Code = Code("E0207");

    pub const WRONG_DOMAIN: Code = Code("E0301");
    pub const TYPE_MISMATCH: Code = Code("E0302");
    pub const WRONG_ARG_COUNT: Code = Code("E0303");
    pub const NOT_CALLABLE: Code = Code("E0304");
    pub const NOT_INDEXABLE: Code = Code("E0305");
    /// Reserved: position arithmetic reports as E0309 and lint L0101 today.
    pub const POSITION_ARITHMETIC: Code = Code("E0306");
    pub const CONDITION_NOT_BOOL: Code = Code("E0307");
    pub const OPTIONAL_COMPARE: Code = Code("E0308");
    pub const BAD_OPERANDS: Code = Code("E0309");
    pub const UNKNOWN_ARG_NAME: Code = Code("E0310");
    /// `@memo` on a fn whose answers can't be remembered.
    pub const BAD_MEMO: Code = Code("E0311");
    /// `debug(..)` of something that changes the program's state: the
    /// C++ leaves `debug` out, and the change with it.
    pub const DEBUG_CHANGES: Code = Code("E0312");
    /// A value mentioning a fresh domain escapes its declaring block.
    pub const DOMAIN_ESCAPES: Code = Code("E0313");
    pub const NON_EXHAUSTIVE_MATCH: Code = Code("E0314");

    pub const ASSIGN_TO_LET: Code = Code("E0401");
    pub const MISSING_MUT_AT_CALL: Code = Code("E0402");
    pub const MUT_OF_IMMUTABLE: Code = Code("E0403");
    /// A `var`-bound lambda that calls itself.
    pub const VAR_LAMBDA_RECURSION: Code = Code("E0406");
    pub const DUPLICATE_VARIANT_TAG: Code = Code("E0407");

    pub const UNSUPPORTED: Code = Code("E0501");
    /// A warning: a statement that only works out a value.
    pub const UNUSED_VALUE: Code = Code("W0101");
    pub const NON_ASCII: Code = Code("E0502");
    /// A spelling an earlier version of the language had and v2.2 removed.
    pub const REMOVED_SYNTAX: Code = Code("E0503");
    /// Syntax the 0.3 design has and this version does not implement yet.
    pub const NOT_IMPLEMENTED: Code = Code("E0504");

    pub const LENT_TWICE: Code = Code("E0601");
    pub const USED_WHILE_LENT: Code = Code("E0602");
    pub const LENT_ESCAPES: Code = Code("E0603");
    pub const BAD_PROJECTION: Code = Code("E0604");
    pub const BAD_INDEX_IMPL: Code = Code("E0605");
    pub const MISSING_IMPL: Code = Code("E0606");
    /// A warning: an assignment whose right side changes the place it writes.
    pub const SAME_PLACE_READ_WRITE: Code = Code("L0601");
    /// A warning: a `@cache` field written from outside its struct.
    pub const CACHE_WRITTEN_OUTSIDE: Code = Code("L0609");

    /// A fixed-width integer type name; widths come from ranges.
    pub const NO_NARROW_INT: Code = Code("E0901");

    /// A captured variable moved while a closure or nested fn capturing it is still live.
    pub const MOVED_WHILE_CAPTURED: Code = Code("E0607");
    /// A heap value read after `let b = a` moved it.
    pub const USE_AFTER_MOVE: Code = Code("E0608");
    /// A bare tuple of operators like `(+, max)` instead of `each(+, max)` or `Semiring(max, +)`.
    pub const BARE_OP_TUPLE: Code = Code("E0703");

    pub const AMBIGUOUS_METHOD: Code = Code("E0701");
    /// A context value that is not a compile-time constant. (R09.1.2)
    pub const CONTEXT_NOT_CONSTANT: Code = Code("E0801");
    /// A bare `with` line after another statement of its block. (R09.1.1)
    pub const WITH_NOT_FIRST: Code = Code("E0802");
    pub const QUESTION_IN_CONDITION: Code = Code("E0803");
    pub const INFERENCE_NEEDS_ANNOTATION: Code = Code("E0806");

    pub const RUNTIME: Code = Code("R0001");
    pub const INDEX_OUT_OF_RANGE: Code = Code("R0101");
    pub const DOMAIN_CAST: Code = Code("R0102");
    pub const OVERFLOW: Code = Code("R0202");
    pub const DIVISION_BY_ZERO: Code = Code("R0203");
    pub const UNWRAP_NONE: Code = Code("R0301");
    pub const RANGE_STORE: Code = Code("R0201");
    pub const ASSERTION_FAILED: Code = Code("R0401");
    pub const TODO_REACHED: Code = Code("R0402");
    pub const BAD_INPUT_TOKEN: Code = Code("R0501");
    pub const INPUT_OUT_OF_CONTRACT: Code = Code("R0502");
    pub const STACK_EXHAUSTED: Code = Code("R0601");

    pub const INTERNAL: Code = Code("E9999");
}

/// A secondary span with its own text, shown under the main one.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Label {
    pub span: Span,
    pub message: String,
}

/// A machine-applicable fix: apply every edit and the error is gone.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Fix {
    pub title: String,
    pub edits: Vec<(Span, String)>,
}

/// One problem, reported once at its root cause.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Diagnostic {
    pub code: Code,
    pub severity: Severity,
    pub message: String,
    /// Where the problem is. The caret goes here.
    pub span: Span,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    /// What to type next.
    pub help: Option<String>,
    pub fixes: Vec<Fix>,
}

impl Diagnostic {
    pub fn error(code: Code, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            code,
            severity: Severity::Error,
            message: message.into(),
            span,
            labels: Vec::new(),
            notes: Vec::new(),
            help: None,
            fixes: Vec::new(),
        }
    }

    pub fn warning(code: Code, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            severity: Severity::Warning,
            ..Diagnostic::error(code, span, message)
        }
    }

    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Diagnostic {
        self.labels.push(Label {
            span,
            message: message.into(),
        });
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Diagnostic {
        self.notes.push(note.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Diagnostic {
        self.help = Some(help.into());
        self
    }

    pub fn with_fix(
        mut self,
        title: impl Into<String>,
        span: Span,
        replacement: impl Into<String>,
    ) -> Diagnostic {
        self.fixes.push(Fix {
            title: title.into(),
            edits: vec![(span, replacement.into())],
        });
        self
    }

    /// The same diagnostic with every span moved right by `by` bytes.
    pub fn shift(mut self, by: u32) -> Diagnostic {
        self.span = self.span.shift(by);
        for label in &mut self.labels {
            label.span = label.span.shift(by);
        }
        for fix in &mut self.fixes {
            for edit in &mut fix.edits {
                edit.0 = edit.0.shift(by);
            }
        }
        self
    }
}

/// Collects diagnostics and keeps one per root cause.
///
/// Two rules make "one missing `)` never cascades" hold across phases:
/// a second diagnostic of the same severity at exactly the same span is
/// dropped (the first names the problem there), and anything reported
/// inside a *poisoned* span is dropped. The parser poisons the span of each
/// syntax error it recovers from, so the checker stays quiet about code the
/// parser could not read.
#[derive(Default, Debug, Clone)]
pub struct Sink {
    diags: Vec<Diagnostic>,
    poisoned: Vec<Span>,
}

impl Sink {
    pub fn new() -> Sink {
        Sink::default()
    }

    pub fn with_poisoned(poisoned: Vec<Span>) -> Sink {
        Sink {
            diags: Vec::new(),
            poisoned,
        }
    }

    pub fn poison(&mut self, span: Span) {
        self.poisoned.push(span);
    }

    pub fn is_poisoned(&self, span: Span) -> bool {
        self.poisoned
            .iter()
            .any(|p| p.contains(span) || (span.is_empty() && p.touches(span.start)))
    }

    /// Whether `span` takes in code the parser could not read, so an error
    /// about all of it, like a call's argument count, is that error's echo.
    pub fn encloses_poison(&self, span: Span) -> bool {
        self.poisoned.iter().any(|p| span.contains(*p))
    }

    pub fn push(&mut self, diag: Diagnostic) {
        if self.is_poisoned(diag.span) {
            return;
        }
        if self
            .diags
            .iter()
            .any(|d| d.span == diag.span && (d.code == diag.code || d.severity == diag.severity))
        {
            return;
        }
        self.diags.push(diag);
    }

    pub fn is_empty(&self) -> bool {
        self.diags.is_empty()
    }

    pub fn len(&self) -> usize {
        self.diags.len()
    }

    /// The diagnostic pushed last, to add help or labels to.
    pub fn last_mut(&mut self) -> Option<&mut Diagnostic> {
        self.diags.last_mut()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diags
    }

    pub fn poisoned(&self) -> &[Span] {
        &self.poisoned
    }

    /// The diagnostics in source order. A feature that is not implemented
    /// yet is one root cause however often the file uses it, so only its
    /// first use is kept.
    pub fn finish(mut self) -> Vec<Diagnostic> {
        self.diags.sort_by_key(|d| (d.span.start, d.span.end));
        let mut planned: Vec<String> = Vec::new();
        self.diags.retain(|d| {
            if d.code != codes::NOT_IMPLEMENTED {
                return true;
            }
            if planned.contains(&d.message) {
                return false;
            }
            planned.push(d.message.clone());
            true
        });
        self.diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_is_named() {
        assert_eq!(NAME, "cpl-diag");
    }

    #[test]
    fn spans_join_shift_and_touch() {
        let a = Span::new(2, 5);
        let b = Span::new(9, 12);
        assert_eq!(a.to(b), Span::new(2, 12));
        assert_eq!(a.shift(10), Span::new(12, 15));
        assert!(a.touches(5) && a.touches(2) && !a.touches(6));
        assert_eq!(a.text("0123456789"), "234");
        assert!(Span::point(3).is_empty());
    }

    #[test]
    fn sink_drops_repeats_of_the_same_root_cause() {
        let mut sink = Sink::new();
        let d = Diagnostic::error(codes::UNRESOLVED_NAME, Span::new(4, 7), "no `foo` in scope");
        sink.push(d.clone());
        sink.push(d);
        assert_eq!(sink.len(), 1);
    }

    #[test]
    fn sink_drops_anything_inside_a_poisoned_span() {
        let mut sink = Sink::new();
        sink.poison(Span::new(10, 30));
        sink.push(Diagnostic::error(
            codes::TYPE_MISMATCH,
            Span::new(12, 15),
            "inside",
        ));
        sink.push(Diagnostic::error(
            codes::TYPE_MISMATCH,
            Span::new(40, 42),
            "outside",
        ));
        let out = sink.finish();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].message, "outside");
    }

    #[test]
    fn finish_orders_by_position() {
        let mut sink = Sink::new();
        sink.push(Diagnostic::error(
            codes::TYPE_MISMATCH,
            Span::new(40, 42),
            "second",
        ));
        sink.push(Diagnostic::error(
            codes::UNRESOLVED_NAME,
            Span::new(1, 2),
            "first",
        ));
        let out = sink.finish();
        assert_eq!(out[0].message, "first");
        assert_eq!(out[1].message, "second");
    }

    #[test]
    fn shifting_a_diagnostic_moves_every_span() {
        let d = Diagnostic::error(codes::ASSIGN_TO_LET, Span::new(1, 2), "m")
            .with_label(Span::new(0, 1), "declared here")
            .with_fix("make it a var", Span::new(0, 3), "var")
            .shift(100);
        assert_eq!(d.span, Span::new(101, 102));
        assert_eq!(d.labels[0].span, Span::new(100, 101));
        assert_eq!(d.fixes[0].edits[0].0, Span::new(100, 103));
    }
}

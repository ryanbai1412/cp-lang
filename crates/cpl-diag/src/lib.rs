//! Diagnostics: source spans, stable error codes, and rendering shared by every tool.
//!
//! Every tool (`cpl check`, `cpl run`, `cpl build`, `cpl lsp`) reports through
//! the types here, so an error reads the same on the command line and in the
//! editor (`plan/30-compiler.md` 3.7).

mod fix;
pub mod habits;
pub mod json;
pub mod msgs;
pub mod plain;
pub mod registry;
mod render;
mod source;
pub mod usage;
pub mod view;

pub use fix::{import_edit, splice, splice_fixes};
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

    /// The span's place in source order, by `start` then `end`, as one
    /// [`sort_by_key`] key.
    pub const fn order(self) -> u64 {
        (self.start as u64) << 32 | self.end as u64
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
    /// leaves them untouched; this turns them back into file positions. It
    /// wraps: `shift(d.wrapping_neg())` moves a span `d` bytes left.
    pub const fn shift(self, by: u32) -> Span {
        Span::new(self.start.wrapping_add(by), self.end.wrapping_add(by))
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
/// Sorts `v` stably by a `u64` key, such as [`Span::order`]. The sort itself is one
/// function for every caller: a std `sort_by_key` copies the whole stable sort, about 4 KB
/// of wasm, into each call site whose element holds a `Span`.
pub fn sort_by_key<T>(v: &mut [T], key: impl Fn(&T) -> u64) {
    let order = sorted_order(v.iter().map(key).collect());
    permute(order, &mut |i, j| v.swap(i, j));
}

/// Sorts `v` stably by a string key, through the same one sort as [`sort_by_key`]: the
/// keys are ranked by one `sort_unstable` of `&str` shared by every caller.
pub fn sort_by_str_key<T>(v: &mut [T], key: impl Fn(&T) -> &str) {
    let order = str_order(v.iter().map(key).collect());
    permute(order, &mut |i, j| v.swap(i, j));
}

fn str_order(keys: Vec<&str>) -> Vec<u32> {
    let mut ranks = keys.clone();
    ranks.sort_unstable();
    ranks.dedup();
    sorted_order(
        keys.iter()
            .map(|k| ranks.partition_point(|r| r < k) as u64)
            .collect(),
    )
}

/// The indices of `keys`, sorted by key and then by index, so that equal keys keep their
/// order. Each key is split into its high and low halves so the sort is std's
/// `sort_unstable` of `(u32, u32, u32)`, which cpl.wasm already has from cpl-check, and so
/// adds no code to `core`.
fn sorted_order(keys: Vec<u64>) -> Vec<u32> {
    let mut at: Vec<(u32, u32, u32)> = keys
        .into_iter()
        .zip(0..)
        .map(|(k, i)| ((k >> 32) as u32, k as u32, i))
        .collect();
    at.sort_unstable();
    at.into_iter().map(|(_, _, i)| i).collect()
}

/// Puts the element at `order[i]` at `i`, one cycle at a time.
fn permute(mut order: Vec<u32>, swap: &mut dyn FnMut(usize, usize)) {
    for start in 0..order.len() {
        let mut at = start;
        while order[at] != u32::MAX {
            let from = order[at] as usize;
            order[at] = u32::MAX;
            if from == start {
                break;
            }
            swap(at, from);
            at = from;
        }
    }
}

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
    /// The `else` block of `let p = e else:` can reach its end. (R01.6.2)
    pub const ELSE_FALLS_THROUGH: Code = Code("E0112");

    pub const UNRESOLVED_NAME: Code = Code("E0201");
    pub const DUPLICATE_DEFINITION: Code = Code("E0202");
    pub const UNKNOWN_FIELD: Code = Code("E0203");
    pub const UNKNOWN_TYPE: Code = Code("E0204");
    pub const USED_BEFORE_SET: Code = Code("E0205");
    pub const UNRESOLVED_MODULE: Code = Code("E0206");
    pub const PRIVATE_NAME: Code = Code("E0207");
    pub const VALUE_NAME_CASE: Code = Code("E0208");
    pub const DEFINITION_CYCLE: Code = Code("E0209");
    /// `@intrinsic("id")` names no table entry, or not with this signature. (R12.14.1)
    pub const UNKNOWN_INTRINSIC: Code = Code("E0210");
    /// Working out a `const` at compile time took more steps than the fixed budget.
    pub const CONST_BUDGET: Code = Code("E0211");

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
    /// `read<Bool>()`: `Bool` is not an input token (R10.2.3).
    pub const NOT_A_TOKEN_TYPE: Code = Code("E0315");
    /// `x: 0..=n` in a struct with `n` read: layout never depends on input (R03.1.3).
    pub const FIELD_BOUND_FROM_INPUT: Code = Code("E0316");
    /// `fn f(c: 1..=m)` with `var m`: only a top-level `let` may end a type (R02.10.3).
    pub const RANGE_END_VAR: Code = Code("E0326");
    /// A `@memo_table(word)` struct with no impl of `MemoTable`: nothing fills the table's `get`
    /// and `set` (R06.4.1).
    pub const MEMO_TABLE_WITHOUT_IMPL: Code = Code("E0327");
    /// `@get fn f(self, k: Int)`: a getter's parameters all need defaults (R02.6.6).
    pub const GETTER_NEEDS_DEFAULTS: Code = Code("E0317");
    /// A `match` arm after one that matches everything (R01.7.3).
    pub const UNREACHABLE_ARM: Code = Code("E0318");
    /// A generic fn that calls itself at a type that grows each time (`f([x])` in `f<T>`).
    pub const GROWING_INSTANTIATION: Code = Code("E0319");
    /// `pre[k]` with an `Int` `k` into a cut table on `Int` keys (R07.1.6, decision 507).
    pub const INT_INDEX_ON_CUT_TABLE: Code = Code("E0320");
    /// `a[-1]`: a negative literal key, which never wraps (R07.2.17, decision 508).
    pub const NEGATIVE_LITERAL_INDEX: Code = Code("E0321");
    /// A struct or enum field that stores the type itself (`l: ?N` in `N`).
    pub const RECURSIVE_FIELD: Code = Code("E0322");
    pub const ALIAS_CYCLE: Code = Code("E0323");
    /// `type_of(x)` of a name that is not a `fn` or an operator (R08.1.10).
    pub const NOT_A_FN_ITEM: Code = Code("E0324");
    /// An `@stress` that says no kind, or is not on a type alias (decision 25.32).
    pub const BAD_STRESS: Code = Code("E0325");

    pub const ASSIGN_TO_LET: Code = Code("E0401");
    pub const MISSING_MUT_AT_CALL: Code = Code("E0402");
    pub const MUT_OF_IMMUTABLE: Code = Code("E0403");
    pub const ASSIGN_FIXED_FIELD: Code = Code("E0405");
    /// A `var`-bound lambda that calls itself.
    pub const VAR_LAMBDA_RECURSION: Code = Code("E0406");
    pub const DUPLICATE_VARIANT_TAG: Code = Code("E0407");
    pub const ASSIGN_INTO_RANGE: Code = Code("E0408");
    pub const FN_FALLS_THROUGH: Code = Code("E0409");
    /// A `return`, `break` or `continue` inside a value that is still being built.
    pub const LEAVES_A_PART: Code = Code("E0410");

    pub const UNSUPPORTED: Code = Code("E0501");
    /// A warning: a statement that only works out a value.
    pub const UNUSED_VALUE: Code = Code("W0101");
    /// A warning: the C++ for the judge is past its size target.
    pub const SIZE_NEAR_LIMIT: Code = Code("W0201");
    pub const NON_ASCII: Code = Code("E0502");
    /// A spelling an earlier version of the language had and v2.2 removed.
    pub const REMOVED_SYNTAX: Code = Code("E0503");
    /// Syntax the 0.3 design has and this version does not implement yet.
    pub const NOT_IMPLEMENTED: Code = Code("E0504");
    /// An `@lint` a library fn declares that cannot be used (R06.4.9).
    pub const BAD_LIBRARY_LINT: Code = Code("E0505");

    pub const AMBIGUOUS_METHOD: Code = Code("E0701");
    /// `(+, max)`: a tuple of operators is not an operation.
    pub const BARE_OP_TUPLE: Code = Code("E0703");
    /// A record missing a field a row bound `(w: W, ..)` requires.
    pub const ROW_MISSING_FIELD: Code = Code("E0704");
    /// An argument a law-specialized structure does not take (`Fenwick(act:)`).
    pub const LAW_UNSUPPORTED_SPECIALIZATION: Code = Code("E0705");
    pub const EMPTY_FOLD_NO_IDENTITY: Code = Code("E0706");

    pub const LENT_TWICE: Code = Code("E0601");
    pub const USED_WHILE_LENT: Code = Code("E0602");
    pub const LENT_ESCAPES: Code = Code("E0603");
    pub const BAD_PROJECTION: Code = Code("E0604");
    pub const BAD_INDEX_IMPL: Code = Code("E0605");
    pub const MISSING_IMPL: Code = Code("E0606");
    /// A warning: an assignment whose right side changes the place it writes.
    pub const SAME_PLACE_READ_WRITE: Code = Code("L0601");
    pub const CHAIN_WALKED_TWICE: Code = Code("L0602");
    pub const MAP_RECOMPUTED: Code = Code("L0603");
    pub const MUT_GENERATOR_REWALKED: Code = Code("L0604");
    pub const LITERAL_SLICE_INDEX: Code = Code("L0605");
    pub const WIDE_ARRAY: Code = Code("L0606");
    pub const PADDING: Code = Code("L0607");
    pub const PARENS_MEAN_WORK: Code = Code("L0608");
    pub const CACHE_FIELD_WRITTEN_OUTSIDE: Code = Code("L0609");
    pub const MUT_PARAM_UNUSED: Code = Code("L0610");

    /// A fixed-width integer type name; widths come from ranges.
    pub const NO_NARROW_INT: Code = Code("E0901");
    pub const RANGE_STORE_CONST: Code = Code("E0902");
    /// The peak at the input bounds is over the judge's memory limit. (R03.1.9)
    pub const MEMORY_OVER_PROFILE: Code = Code("E0904");

    /// A captured variable moved while a closure or nested fn capturing it is still live.
    pub const MOVED_WHILE_CAPTURED: Code = Code("E0607");
    /// A heap value read after `let b = a` moved it.
    pub const USE_AFTER_MOVE: Code = Code("E0608");
    pub const OVERLOAD_BY_TYPE: Code = Code("E0702");

    /// A position of a per-test domain kept in a name declared above `@!multitest`. (R02.9.2)
    pub const FRESH_TYPE_LEAKS_TEST: Code = Code("E0805");
    /// A context value that is not a compile-time constant. (R09.1.2)
    pub const CONTEXT_NOT_CONSTANT: Code = Code("E0801");
    /// A bare `with` line after another statement of its block. (R09.1.1)
    pub const WITH_NOT_FIRST: Code = Code("E0802");
    /// `x?` in an `if` or `while` condition.
    pub const QUESTION_IN_CONDITION: Code = Code("E0803");
    /// A statement that is just `x?`: its result goes nowhere. (R02.4.7)
    pub const DROPPED_QUESTION: Code = Code("E0804");
    /// `var x = None` or `let x = read()`: nothing local says the type.
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
    /// A part of the program with a type error ran, in a run despite its errors.
    pub const UNTYPED_RAN: Code = Code("R0403");
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
    /// Input values that make the problem happen (`n = 0`): `cpl lint
    /// --save` writes a test case from them.
    pub witness: Vec<(String, i128)>,
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
            witness: Vec::new(),
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

    /// "did you mean `name`?", with the change at `span` as the fix.
    pub fn suggest(&mut self, span: Span, name: impl Into<String>) {
        let name = name.into();
        self.help = Some(format!("did you mean `{name}`?"));
        self.fixes.push(Fix {
            title: format!("change to `{name}`"),
            edits: vec![(span, name)],
        });
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Diagnostic {
        self.help = Some(help.into());
        self
    }

    pub fn with_witness(mut self, witness: Vec<(String, i128)>) -> Diagnostic {
        self.witness = witness;
        self
    }

    pub fn with_fix(
        mut self,
        title: impl Into<String>,
        span: Span,
        replacement: impl Into<String>,
    ) -> Diagnostic {
        self.fix(title, span, replacement);
        self
    }

    /// Adds a fix that replaces `span` with `replacement`.
    /// A fix of several edits, applied together.
    pub fn with_edits(
        mut self,
        title: impl Into<String>,
        edits: Vec<(Span, String)>,
    ) -> Diagnostic {
        self.fixes.push(Fix {
            title: title.into(),
            edits,
        });
        self
    }

    pub fn fix(&mut self, title: impl Into<String>, span: Span, replacement: impl Into<String>) {
        let edits = vec![(span, replacement.into())];
        self.fixes.push(Fix {
            title: title.into(),
            edits,
        });
    }

    /// Fix text written with `\n` line breaks, made to use `src`'s own:
    /// a fix applied to a CRLF file keeps it CRLF.
    pub fn match_line_endings(&mut self, src: &str) {
        for fix in &mut self.fixes {
            for edit in &mut fix.edits {
                *edit = edit_like(src, edit.0, &edit.1);
            }
        }
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

/// An edit of `src` with `src`'s line endings. Fixes are written as if
/// lines end in `\n`; in a CRLF file a span boundary between the `\r` and
/// `\n` of a line break moves before the `\r`, and `text` uses `\r\n`.
pub fn edit_like(src: &str, span: Span, text: &str) -> (Span, String) {
    if !src.contains("\r\n") {
        return (span, text.to_string());
    }
    let before_break = |at: u32| {
        let at = at as usize;
        if at > 0 && src.get(at - 1..at + 1) == Some("\r\n") {
            at as u32 - 1
        } else {
            at as u32
        }
    };
    let span = Span::new(before_break(span.start), before_break(span.end));
    (span, text.replace('\r', "").replace('\n', "\r\n"))
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

    /// Drops a diagnostic a later one explains, keeping one per root cause.
    pub fn drop_where(&mut self, f: impl Fn(&Diagnostic) -> bool) {
        self.diags.retain(|d| !f(d));
    }

    pub fn poisoned(&self) -> &[Span] {
        &self.poisoned
    }

    /// The diagnostics in source order. A feature that is not implemented
    /// yet, or a name nothing declares, is one root cause however often the
    /// file uses it, so only its first use is kept; a later use's fix, the
    /// same rename, joins the first one's edits.
    pub fn finish(mut self) -> Vec<Diagnostic> {
        sort_by_key(&mut self.diags, |d| d.span.order());
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
        let mut kept: Vec<Diagnostic> = Vec::with_capacity(self.diags.len());
        let mut first_use: Vec<(String, usize)> = Vec::new();
        for diag in self.diags {
            let Some(name) = unresolved_name(&diag).map(str::to_string) else {
                kept.push(diag);
                continue;
            };
            let Some(&(_, at)) = first_use.iter().find(|(n, _)| *n == name) else {
                first_use.push((name, kept.len()));
                kept.push(diag);
                continue;
            };
            let first = &mut kept[at];
            for fix in diag.fixes {
                if let Some(same) = first.fixes.iter_mut().find(|f| f.title == fix.title) {
                    for edit in fix.edits {
                        if !same.edits.contains(&edit) {
                            same.edits.push(edit);
                        }
                    }
                }
            }
        }
        kept
    }
}

/// The name an unresolved-name or unknown-type error is about: `X` in
/// "no `X` in scope", "no type or domain named `X`", "`std.m` has no `X`".
fn unresolved_name(diag: &Diagnostic) -> Option<&str> {
    if diag.code != codes::UNRESOLVED_NAME && diag.code != codes::UNKNOWN_TYPE {
        return None;
    }
    let m = &diag.message;
    let rest = m
        .strip_prefix("no type or domain named `")
        .or_else(|| m.strip_prefix("no `"))
        .or_else(|| {
            m.starts_with('`')
                .then(|| m.split_once("` has no `"))
                .flatten()
                .map(|(_, r)| r)
        })?;
    rest.split_once('`').map(|(name, _)| name)
}

#[cfg(test)]
mod tests {
    #[test]
    fn sort_by_key_is_stable_and_matches_std() {
        let mut seed = 7u64;
        for n in 0..60 {
            let v: Vec<(u64, usize)> = (0..n)
                .map(|i| {
                    seed = seed
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    (seed >> 61, i)
                })
                .collect();
            let (mut ours, mut std) = (v.clone(), v);
            super::sort_by_key(&mut ours, |p| p.0);
            std.sort_by_key(|p| p.0);
            assert_eq!(ours, std);
        }
    }

    #[test]
    fn sort_by_str_key_is_stable_and_matches_std() {
        let words = ["b", "", "ab", "b", "a", "ab", "ba", "a"];
        let v: Vec<(&str, usize)> = words.iter().copied().zip(0..).collect();
        let (mut ours, mut std) = (v.clone(), v);
        super::sort_by_str_key(&mut ours, |p| p.0);
        std.sort_by_key(|p| p.0);
        assert_eq!(ours, std);
    }

    use super::*;

    #[test]
    fn an_insertion_at_another_fixs_start_goes_before_its_replacement() {
        let src = "ab";
        let replace = Diagnostic::error(codes::UNRESOLVED_NAME, Span::new(0, 2), "r").with_fix(
            "replace",
            Span::new(0, 2),
            "xy",
        );
        let insert = Diagnostic::error(codes::UNRESOLVED_NAME, Span::new(0, 0), "i").with_fix(
            "insert",
            Span::new(0, 0),
            "(",
        );
        assert_eq!(splice_fixes(src, &[replace, insert]), ("(xy".into(), 2));
        // One fix whose own edits overlap is a producer bug, left alone.
        let mut both = Diagnostic::error(codes::UNRESOLVED_NAME, Span::new(0, 2), "b");
        both.fixes.push(Fix {
            title: "both".into(),
            edits: vec![(Span::new(0, 2), "x".into()), (Span::new(1, 2), "y".into())],
        });
        assert_eq!(splice_fixes(src, &[both]), ("ab".into(), 0));
    }

    #[test]
    fn an_edit_of_a_crlf_file_writes_crlf_and_keeps_whole_line_breaks() {
        let lf = "a\nb\n";
        assert_eq!(
            edit_like(lf, Span::new(1, 1), "\nx"),
            (Span::new(1, 1), "\nx".into())
        );
        let crlf = "a\r\nb\r\n";
        // Inserting a line after `a`, written as if the span were LF-based.
        assert_eq!(
            edit_like(crlf, Span::new(2, 2), "\nx"),
            (Span::new(1, 1), "\r\nx".into())
        );
        // A statement span that took the `\r` of its line break.
        assert_eq!(
            edit_like(crlf, Span::new(0, 2), "c"),
            (Span::new(0, 1), "c".into())
        );
        // Text copied from the CRLF source is not doubled.
        assert_eq!(
            edit_like(crlf, Span::new(3, 4), "b\r\nd"),
            (Span::new(3, 4), "b\r\nd".into())
        );
    }

    #[test]
    fn a_name_nothing_declares_is_one_error_at_its_first_use() {
        let mut sink = Sink::new();
        let err = |at: u32, msg: &str| {
            Diagnostic::error(codes::UNRESOLVED_NAME, Span::new(at, at + 1), msg)
        };
        sink.push(err(9, "no `t` in scope"));
        sink.push(err(2, "no `t` in scope"));
        sink.push(Diagnostic::error(
            codes::UNKNOWN_TYPE,
            Span::new(5, 6),
            "no type or domain named `t`",
        ));
        sink.push(err(7, "no `u` in scope"));
        let rename = |at: u32| {
            err(at, "no `prnt` in scope").with_fix(
                "change to `print`",
                Span::new(at, at + 1),
                "print",
            )
        };
        sink.push(rename(20));
        sink.push(rename(30));
        let kept = sink.finish();
        let messages: Vec<&str> = kept.iter().map(|d| d.message.as_str()).collect();
        assert_eq!(
            messages,
            ["no `t` in scope", "no `u` in scope", "no `prnt` in scope"]
        );
        assert_eq!(kept[0].span, Span::new(2, 3));
        assert_eq!(kept[2].fixes[0].edits.len(), 2);
        assert_eq!(
            unresolved_name(&err(0, "`std.seq` has no `prefix_fold`")),
            Some("prefix_fold")
        );
    }

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

/// A path as `cpl` prints it and hands it to other programs: `/` between
/// folders on every OS (as git spells them), and without the `\\?\` prefix
/// Windows' canonical paths carry, which a shell or a person can't use.
pub fn shown_path(path: &std::path::Path) -> String {
    let text = path.display().to_string();
    if !cfg!(windows) {
        return text;
    }
    // `\\?\UNC\server\share\x` is the share path `\\server\share\x`.
    let plain = match text.strip_prefix(r"\\?\UNC\") {
        Some(share) => format!(r"\\{share}"),
        None => text.strip_prefix(r"\\?\").unwrap_or(&text).to_string(),
    };
    plain.replace('\\', "/")
}

/// `path.shown()` is [`shown_path`] as a method, for messages.
pub trait Shown {
    fn shown(&self) -> String;
}

impl Shown for std::path::Path {
    fn shown(&self) -> String {
        shown_path(self)
    }
}

#[cfg(test)]
mod shown_path_tests {
    #[test]
    fn a_path_is_shown_with_slashes() {
        let shown = super::shown_path(std::path::Path::new("a.tests/c.cp"));
        assert_eq!(shown, "a.tests/c.cp");
        #[cfg(windows)]
        assert_eq!(
            super::shown_path(std::path::Path::new(r"\\?\D:\a\gen.sh")),
            "D:/a/gen.sh"
        );
        #[cfg(windows)]
        assert_eq!(
            super::shown_path(std::path::Path::new(r"\\?\UNC\srv\share\gen.sh")),
            "//srv/share/gen.sh"
        );
    }
}

/// "a V", "an E", "an Int": a type as shown, with its article, for messages.
pub fn with_article(text: &str) -> String {
    // "an F64": the name is said "eff".
    let vowel = text == "F64"
        || text
            .trim_start_matches(['[', '?', '('])
            .starts_with(['A', 'E', 'I', 'O', 'U', 'a', 'e', 'i', 'o', 'u']);
    format!("{} {text}", if vowel { "an" } else { "a" })
}

/// The candidate closest to `name`, if it is close enough to be a typo.
pub fn closest<'c>(name: &str, mut candidates: impl Iterator<Item = &'c str>) -> Option<String> {
    closest_of(name, &mut candidates)
}

/// `closest`'s body, behind `dyn` so its two dozen callers share one copy.
fn closest_of(name: &str, candidates: &mut dyn Iterator<Item = &str>) -> Option<String> {
    let limit = match name.chars().count() {
        0 | 1 => return None,
        2..=4 => 1,
        _ => 2,
    };
    // A value named like a type (`cnt` for `Int`) is a different name, not
    // a slip, unless only the capital differs (`vec` for `Vec`).
    let upper = |s: &str| s.chars().next().is_some_and(char::is_uppercase);
    let head = |s: &str| s.chars().next().map(|c| c.to_ascii_lowercase());
    let mut best: Option<(usize, bool, &str)> = None;
    for c in candidates {
        if c == name || (upper(c) != upper(name) && head(c) != head(name)) {
            continue;
        }
        let mut d = edit_distance(name, c);
        // `pri` is `print` stopped short, a closer match than `prim` is a typo.
        if c.starts_with(name) && name.len() >= 3 && c.len() <= name.len() + 2 {
            d = d.min(limit);
        }
        // A word stopped short (`sorte`) means the longer name.
        let key = (d, !c.starts_with(name), c);
        if d <= limit && best.is_none_or(|b| key < b) {
            best = Some(key);
        }
    }
    best.map(|(_, _, c)| c.to_string())
}

/// Edits to turn `a` into `b`, counting a swap of two neighbours as one:
/// `raods` is one edit from `roads`.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut rows = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in rows.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in rows[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let substitute = rows[i - 1][j - 1] + usize::from(a[i - 1] != b[j - 1]);
            let mut best = substitute.min(rows[i - 1][j] + 1).min(rows[i][j - 1] + 1);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(rows[i - 2][j - 2] + 1);
            }
            rows[i][j] = best;
        }
    }
    rows[a.len()][b.len()]
}

/// The old spelling of an optional in `text`, `none` or `some(`, if any. The
/// spec spells them `None` and `Some(x)`; help, notes and lint records teach
/// that spelling. `none:`, the argument name of `print(xs, none: d)`, is not one.
pub fn old_option_spelling(text: &str) -> Option<&'static str> {
    let ident = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
    text.char_indices().find_map(|(i, _)| {
        if ident(text[..i].chars().next_back()) {
            return None;
        }
        let rest = &text[i..];
        if rest.starts_with("some(") {
            return Some("some(");
        }
        let after = rest.strip_prefix("none")?.chars().next();
        (!ident(after) && after != Some(':')).then_some("none")
    })
}

/// [`old_option_spelling`] in the code of prose: its `` `quoted` `` parts, so
/// the English "there is none" stays.
pub fn old_option_spelling_in_prose(text: &str) -> Option<&'static str> {
    text.split('`')
        .skip(1)
        .step_by(2)
        .find_map(old_option_spelling)
}

#[cfg(test)]
mod option_spelling_tests {
    use super::{old_option_spelling, old_option_spelling_in_prose};

    #[test]
    fn finds_none_and_some_but_not_the_none_argument() {
        assert_eq!(old_option_spelling("`x ?? none`"), Some("none"));
        assert_eq!(old_option_spelling("x == some(3)"), Some("some("));
        assert_eq!(
            old_option_spelling("`print(xs, none: -1)`, `None`, `Some(x)`"),
            None
        );
        assert_eq!(old_option_spelling("nonempty, is_none(), awesome(x)"), None);
        assert_eq!(
            old_option_spelling_in_prose("there is none: `x ?? none`"),
            Some("none")
        );
        assert_eq!(
            old_option_spelling_in_prose("there is none, `x ?? None`"),
            None
        );
    }
}

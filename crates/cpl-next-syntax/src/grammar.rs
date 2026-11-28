//! The grammar's spellings: every keyword and foreign keyword the lexer
//! knows by its text, in one table, so no other code compares source text
//! against a fixed word.

use crate::token::TokenKind::{self, *};

/// Each keyword and the token it lexes to.
pub const KEYWORD_KINDS: &[(&str, TokenKind)] = &[
    ("let", Let),
    ("var", Var),
    ("fn", Fn),
    ("struct", Struct),
    ("enum", Enum),
    ("domain", Domain),
    ("type", Type),
    ("const", Const),
    ("trait", Trait),
    ("impl", Impl),
    ("use", Use),
    ("pub", Pub),
    ("if", If),
    ("elif", Elif),
    ("else", Else),
    ("for", For),
    ("while", While),
    ("in", In),
    ("match", Match),
    ("return", Return),
    ("break", Break),
    ("continue", Continue),
    ("and", And),
    ("or", Or),
    ("not", Not),
    ("assert", Assert),
    ("mut", Mut),
    ("self", SelfKw),
    ("true", True),
    ("false", False),
    ("lends", Lends),
    ("yield", Yield),
    ("unchecked", Unchecked),
];

/// What to write instead of a word that v2.1 reserved and v2.2 does not.
/// The word itself is an ordinary name now.
pub const REMOVED_KEYWORDS: &[(&str, &str)] = &[
    (
        "defer",
        "undo by hand after the call: `cur.push(x); dfs(); cur.pop()`",
    ),
    ("assume", "write `assert`, which is always checked"),
    (
        "where",
        "write the bound in the angle brackets: `fn f<T: Ord>(...)`",
    ),
    (
        "inline",
        "drop it: closures and `lends` bodies are always inlined",
    ),
    ("law", "write `@law name(args) = condition`"),
];

/// Another language's keyword starting a line, and the keyword it is read
/// as so the uses parse with no cascade: `def f(` is `fn f(`.
pub const FOREIGN_KEYWORDS: &[(&str, TokenKind)] = &[
    ("def", Fn),
    ("func", Fn),
    ("function", Fn),
    ("import", Use),
    ("switch", Match),
];

/// A word that is an ordinary name except in one place, where the parser
/// reads it as grammar: `lo..hi by 2`, `x is Some(v)`, `best max= x`, `_`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Contextual {
    By,
    Is,
    Max,
    Min,
    /// `_`, the pattern that binds nothing.
    Wildcard,
    /// `repeat k:` runs its block `k` times.
    Repeat,
    /// `with ctx(..):` runs its block in a context.
    With,
    Mod,
    Extern,
    Where,
    /// `-> yields T`: a generator.
    Yields,
    /// A trait's or impl's `law`; elsewhere `law` is a removed keyword.
    Law,
    /// `context io:` declares a context.
    Context,
    /// `Str of "ab"`: a string over an alphabet.
    Of,
    /// `-> T uses io`: the contexts a fn reads.
    Uses,
    /// `use a.b as c`.
    As,
    /// `@intrinsic("name")`.
    Intrinsic,
    /// `@cpp("code")`.
    Cpp,
    /// `lend mut place`: a `lends` body hands out a place (R05.6.3).
    Lend,
    /// `loop:`, reported: write `while true:`.
    Loop,
    /// `..R without (u, v)`: a row with fields removed.
    Without,
    /// `type_of(f)`: the type of a fn or an operator.
    TypeOf,
    /// `lazy f: T`, reported: a param is lent, `mut` or `var` (R05.2.3).
    Lazy,
    Guard,
    Trusted,
    Do,
    /// `yield from xs` yields each value of `xs`.
    From,
}

/// Each contextual word and how it is spelled.
pub const CONTEXTUAL: &[(&str, Contextual)] = &[
    ("by", Contextual::By),
    ("is", Contextual::Is),
    ("max", Contextual::Max),
    ("min", Contextual::Min),
    ("_", Contextual::Wildcard),
    ("repeat", Contextual::Repeat),
    ("with", Contextual::With),
    ("mod", Contextual::Mod),
    ("extern", Contextual::Extern),
    ("where", Contextual::Where),
    ("yields", Contextual::Yields),
    ("law", Contextual::Law),
    ("context", Contextual::Context),
    ("of", Contextual::Of),
    ("uses", Contextual::Uses),
    ("as", Contextual::As),
    ("intrinsic", Contextual::Intrinsic),
    ("cpp", Contextual::Cpp),
    ("lend", Contextual::Lend),
    ("loop", Contextual::Loop),
    ("lazy", Contextual::Lazy),
    ("guard", Contextual::Guard),
    ("trusted", Contextual::Trusted),
    ("do", Contextual::Do),
    ("from", Contextual::From),
    ("without", Contextual::Without),
    ("type_of", Contextual::TypeOf),
];

/// The contextual word `text` spells, if any.
pub fn contextual(text: &str) -> Option<Contextual> {
    CONTEXTUAL.iter().find(|(w, _)| *w == text).map(|(_, c)| *c)
}

/// The facts a doc comment states, each on its own `label: text` line, in the
/// order std/README.md lists them.
pub const DOC_FACTS: [&str; 5] = ["cost", "traps", "empty", "ties", "order"];

/// The decorator table: each row is a variant, its spelling, the spec rule
/// that defines it and, for the live ones and the directives, what it does.
macro_rules! decorators {
    (
        live: [$($v:ident = $name:literal, $rule:literal, $doc:literal;)*]
        directives: [$($dv:ident = $dname:literal, $drule:literal, $ddoc:literal;)*]
        removed: [$($rv:ident = $rname:literal, $rrule:literal;)*]
    ) => {
        /// A decorator of R06.4.11's closed set, a file directive (`@!name`),
        /// or a removed spelling that gets its own E0503. The name is read
        /// here once; later passes match the variant.
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
        pub enum Decorator {
            $($v,)*
            $($dv,)*
            $($rv,)*
        }

        /// Each live decorator with what it does, as completion shows it.
        pub const DECORATORS: &[(&str, &str)] = &[$(($name, $doc),)*];

        /// The file directives, `@!name`, with what each does.
        pub const DIRECTIVES: &[(&str, &str)] = &[$(($dname, $ddoc),)*];

        /// Spellings the language removed, each answered with its own E0503.
        pub const REMOVED_DECORATORS: &[&str] = &[$($rname,)*];

        impl Decorator {
            const ROWS: &'static [(Decorator, &'static str, &'static str)] = &[
                $((Decorator::$v, $name, $rule),)*
                $((Decorator::$dv, $dname, $drule),)*
                $((Decorator::$rv, $rname, $rrule),)*
            ];
        }
    };
}

decorators! {
    live: [
    Test = "test", "R06.4.3", "tool role: a test fn `cpl test` runs";
    Alt = "alt", "R06.4.3", "tool role: `@alt(f, caps)`, another implementation of `f`";
    Gen = "gen", "R06.4.3", "tool role: a stress generator";
    Check = "check", "R06.4.3", "tool role: the checker of an output";
    Interactor = "interactor", "R06.4.3", "tool role: the judge side of an interactive problem";
    Memo = "memo", "R06.4.1", "`@memo(array)` / `@memo(map)`: cache a function";
    Get = "get", "R06.4.2", "an O(1) read-only method called without parentheses";
    Cache = "cache", "R06.4.4", "on a struct field: interior mutability";
    Bits = "bits", "R05.10.15", "on a `List<U64>` field: one bit per domain position";
    Trusted = "trusted", "R09.5.4", "`@trusted:` block: the unsafe tier";
    Cpp = "cpp", "R09.5.4", "an item written in C++: the unsafe tier";
    Grader = "grader", "R11.1.3", "`@grader(..)`: a judge entry point";
    Laws = "laws", "R06.4.13", "`@laws(sample: n)`: stress sampling of a type's laws";
    Law = "law", "R05.1.1", "`@law name(args) = condition`: a law of a type or a lent place";
    Tag = "tag", "R02.7.2", "`@tag(lit)` on a variant: the token it reads and prints as";
    Droppable = "droppable", "R06.4.8", "the result may be dropped without the unused-result lint";
    FromOp = "from_op", "R08.1.9", "on a struct's static fn: a bare op passed where the struct is wanted goes through it";
    MemoTable = "memo_table", "R06.4.1", "`@memo_table(word)` on a struct: the table a `@memo(word)` fn keeps";
    Structural = "structural", "R06.4.14", "on a trait: a tuple, record, optional, array or struct has it when each part does";
    Stress = "stress", "R12.13.3", "`@stress(kind)` on a type alias: the shape `cpl stress` draws for it";
    Word = "word", "R06.4.15", "`@word(name)` on a std struct: the lowercase call that builds it";
    Member = "member", "R06.4.16", "`@member(name)` on a std struct: the domain member (`D.name`) whose positions it is";
    Lint = "lint", "R06.4.9", "`@lint(name, from:, to:)` on a `pub fn`: a library's own lint";
    OnUnimplemented = "on_unimplemented", "R08.1.8", "on a trait: the message when a type lacks its impl";
    Aka = "aka", "R12.13.2", "`@aka(name, ..)` on a fn or an `@operator` trait: other languages' names for it, for their fix";
    Was = "was", "R12.13.1", "`@was(Old, ..)`: the names a declaration replaced, for their fix";
    SameKeys = "same_keys", "R04b.1.3", "on a fn: its arguments that are arrays over domains share one domain";
    Index = "index", "R07.4.17", "on a fn: what `s[r]` calls when `r` is not a key of `s`";
    Fast = "fast", "R06.4.12", "`@fast(op)` on a fn with a body: the operation the engines and C++ run for a call";
    Role = "role", "R11.6.1", "on a declaration: the language role it fills";
    Operator = "operator", "R06.4.12", "`@operator(\"op\")` on a trait: the trait an operator calls";
    Literal = "literal", "R06.4.12", "`@literal(\"[]\")` on a trait: the trait a list literal builds through; `\"[;]\"` an array fill";
    Derive = "derive", "R06.4.12", "on a trait: every struct whose fields all have it gets it field by field";
    Coerce = "coerce", "R06.4.12", "on a fn: an implicit conversion where its result type is wanted";
    Pattern = "pattern", "R06.4.12", "on a fn: what a pattern over its type calls";
    Reject = "reject", "R06.4.17", "`@reject(when: T = Bool, \"..\", code: C)` on a defaulted parameter: a call taking the default where `when` holds is that error";
    Input = "input", "R06.4.12", "on a fn: a value it gives is input, so a range it fills is a contract";
    FreeForm = "free_form", "R01.10.1", "on a trait's method with `self`: `f(x, ..)` is `x.f(..)` where the trait is in scope";
    ]
    directives: [
    Multitest = "multitest", "R01.7.5", "file directive: loop over t test cases below, with fresh scope";
    ]
    removed: [
    Brute = "brute", "R06.4.3";
    Checker = "checker", "R06.4.3";
    Allow = "allow", "R09.4.4";
    Tags = "tags", "R02.7.3";
    Interactive = "interactive", "R01.9.1";
    ]
}

/// A member a language role's trait declares, which the compiler reaches by this variant, its
/// spelling kept here once (R11.6.1): `@role(memo_table)`'s `get` and `set` (R06.4.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum RoleMember {
    MemoGet,
    MemoSet,
}

impl RoleMember {
    pub const ALL: [RoleMember; 2] = [RoleMember::MemoGet, RoleMember::MemoSet];

    /// The `@role(name)` whose trait declares it, and its name there.
    pub fn spelling(self) -> (&'static str, &'static str) {
        match self {
            RoleMember::MemoGet => ("memo_table", "get"),
            RoleMember::MemoSet => ("memo_table", "set"),
        }
    }
}

/// Whether a decorator is live, a file directive or a removed spelling.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DecoratorKind {
    Live,
    Directive,
    Removed,
}

impl Decorator {
    /// The decorator, directive or removed spelling `@name` is, if any.
    pub fn of(name: &str) -> Option<Decorator> {
        Self::ROWS.iter().find(|r| r.1 == name).map(|r| r.0)
    }

    /// The live decorator `@name` is: neither a directive nor removed.
    pub fn live(name: &str) -> Option<Decorator> {
        Self::of(name).filter(|d| d.kind() == DecoratorKind::Live)
    }

    pub fn name(self) -> &'static str {
        Self::ROWS.iter().find(|r| r.0 == self).map_or("", |r| r.1)
    }

    /// The spec rule that defines it.
    pub fn rule(self) -> &'static str {
        Self::ROWS.iter().find(|r| r.0 == self).map_or("", |r| r.2)
    }

    pub fn kind(self) -> DecoratorKind {
        let name = self.name();
        if DECORATORS.iter().any(|(n, _)| *n == name) {
            DecoratorKind::Live
        } else if DIRECTIVES.iter().any(|(n, _)| *n == name) {
            DecoratorKind::Directive
        } else {
            DecoratorKind::Removed
        }
    }

    /// Every rule a row cites, for the test that each names a spec rule.
    pub fn rules() -> impl Iterator<Item = &'static str> {
        Self::ROWS.iter().map(|r| r.2)
    }
}

/// A shape `@stress(kind)` says `cpl stress` makes and shrinks a type alias's
/// values as. The word is read here once; later passes match the variant.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum StressKind {
    /// Each value of the index once.
    Perm,
    /// The `V.len - 1` edges of a tree over `V`.
    Tree,
    /// No loop and no repeated unordered pair.
    Simple,
    /// Simple and connected.
    Connected,
    /// Arcs that follow a vertex order.
    Dag,
    /// Arcs with no loop and no repeated arc.
    Digraph,
}

impl StressKind {
    /// Each row: the variant, its spelling and the spec rule that defines it.
    pub const ROWS: [(StressKind, &'static str, &'static str); 6] = [
        (StressKind::Perm, "perm", "R12.13.3"),
        (StressKind::Tree, "tree", "R12.13.3"),
        (StressKind::Simple, "simple", "R12.13.3"),
        (StressKind::Connected, "connected", "R12.13.3"),
        (StressKind::Dag, "dag", "R12.13.3"),
        (StressKind::Digraph, "digraph", "R12.13.3"),
    ];

    /// The kind `@stress(word)` names, if any.
    pub fn of(word: &str) -> Option<StressKind> {
        Self::ROWS.iter().find(|r| r.1 == word).map(|r| r.0)
    }

    pub fn name(self) -> &'static str {
        Self::ROWS.iter().find(|r| r.0 == self).map_or("", |r| r.1)
    }

    /// The spec rule that defines it.
    pub fn rule(self) -> &'static str {
        Self::ROWS.iter().find(|r| r.0 == self).map_or("", |r| r.2)
    }
}

/// A labelled argument of `@lint(name, from: "..", to: "..")`: what the lint
/// matches, what its fix writes and where it applies.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum LintArg {
    /// The code the lint matches.
    From,
    /// The code its fix writes.
    To,
    /// Advice shown in place of a fix.
    Advice,
    /// Names `from` matches besides the fn's parameters.
    Holes,
    /// Statements the match must also find.
    Also,
    /// A read of a matched name, `->`, what the fix writes for it.
    Reads,
    /// The singular `to` writes for one matched name.
    Each,
    /// Bounds on holes the match needs.
    When,
    /// The call the fix suggests when it is not the fn's own.
    Builtin,
}

impl LintArg {
    /// Each row: the variant, its spelling and the spec rule that defines it.
    pub const ROWS: [(LintArg, &'static str, &'static str); 9] = [
        (LintArg::From, "from", "R06.4.9"),
        (LintArg::To, "to", "R06.4.9"),
        (LintArg::Advice, "advice", "R06.4.9"),
        (LintArg::Holes, "holes", "R06.4.9"),
        (LintArg::Also, "also", "R06.4.9"),
        (LintArg::Reads, "reads", "R06.4.9"),
        (LintArg::Each, "each", "R06.4.9"),
        (LintArg::When, "when", "R06.4.9"),
        (LintArg::Builtin, "builtin", "R06.4.9"),
    ];

    /// The argument the label `word:` is, if any.
    pub fn of(word: &str) -> Option<LintArg> {
        Self::ROWS.iter().find(|r| r.1 == word).map(|r| r.0)
    }

    pub fn name(self) -> &'static str {
        Self::ROWS.iter().find(|r| r.0 == self).map_or("", |r| r.1)
    }

    /// The spec rule that defines it.
    pub fn rule(self) -> &'static str {
        Self::ROWS.iter().find(|r| r.0 == self).map_or("", |r| r.2)
    }
}

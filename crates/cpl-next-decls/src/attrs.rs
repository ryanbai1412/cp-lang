//! The decorator vocabulary: what each `@name` an item carries means to the declarations
//! (R11.6). A role a declaration fills is `@role(..)` (`Role`); every other decorator is an
//! attribute here. The name is read once, at the `Decorator` node; past it decls keeps the
//! `Attr`, never the text.

/// A decorator past the parser: what it asks of the declaration it is on.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Attr {
    /// tool role: a test fn `cpl test` runs
    Test,
    /// tool role: `@alt(f, caps)`, another implementation of `f`
    Alt,
    /// tool role: a stress generator
    Gen,
    /// tool role: the checker of an output
    Check,
    /// tool role: the judge side of an interactive problem
    Interactor,
    /// `@memo(array)` / `@memo(map)`: cache a function
    Memo,
    /// an O(1) read-only method called without parentheses
    Get,
    /// on a struct field: interior mutability
    Cache,
    /// on a `List<U64>` field: one bit per domain position
    Bits,
    /// `@trusted:` block: the unsafe tier
    Trusted,
    /// an item written in C++: the unsafe tier
    Cpp,
    /// `@grader(..)`: a judge entry point
    Grader,
    /// `@laws(sample: n)`: stress sampling of a type's laws
    Laws,
    /// `@law name(args) = condition`: a law of a type or a lent place
    Law,
    /// `@tag(lit)` on a variant: the token it reads and prints as
    Tag,
    /// the result may be dropped without the unused-result lint
    Droppable,
    /// on a struct's static fn: a bare op passed where the struct is wanted goes through it
    FromOp,
    /// `@memo_table(word)` on a struct: the table a `@memo(word)` fn keeps
    MemoTable,
    /// on a trait: a tuple, record, optional, array or struct has it when each part does
    Structural,
    /// `@stress(kind)` on a type alias: the shape `cpl stress` draws for it
    Stress,
    /// `@word(name)` on a std struct: the lowercase call that builds it
    Word,
    /// `@member(name)` on a std struct: the domain member (`D.name`) whose positions it is
    Member,
    /// `@lint(name, from:, to:)` on a `pub fn`: a library's own lint
    Lint,
    /// on a trait: the message when a type lacks its impl
    OnUnimplemented,
    /// `@aka(name, ..)` on a fn: other languages' names for it, for their fix
    Aka,
    /// `@was(Old, ..)`: the names a declaration replaced, for their fix
    Was,
    /// on a fn: its arguments that are arrays over domains share one domain (R04b.1.3)
    SameKeys,
    /// on a fn: what `s[r]` calls when `r` is not a key of `s` (R07.4.17)
    Index,
    /// `@fast(op)` on a fn with a body: the operation the engines and C++ run for a call
    Fast,
    /// on a declaration: the language role it fills (R11.6.1)
    Role,
    /// `@operator(\"op\")` on a trait: the trait an operator calls
    Operator,
    /// `@literal(\"[]\")` on a trait: the trait a list literal builds through; `\"[;]\"` an array fill
    Literal,
    /// on a trait: every struct whose fields all have it gets it field by field
    Derive,
    /// on a fn: an implicit conversion where its result type is wanted
    Coerce,
    /// on a fn: what a pattern over its type calls
    Pattern,
    /// on a fn: a value it gives is input, so a range it fills is a contract
    Input,
    /// The compiler's own mark on a record struct it declares; no source writes it.
    Record,
}

/// A file-level `@!name`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Directive {
    Multitest,
    Interactive,
}

/// A decorator the language had and removed, each with its own fix.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Removed {
    Brute,
    Checker,
    Allow,
    Tags,
}

const ATTRS: &[(Attr, &str)] = &[
    (Attr::Test, "test"),
    (Attr::Alt, "alt"),
    (Attr::Gen, "gen"),
    (Attr::Check, "check"),
    (Attr::Interactor, "interactor"),
    (Attr::Memo, "memo"),
    (Attr::Get, "get"),
    (Attr::Cache, "cache"),
    (Attr::Bits, "bits"),
    (Attr::Trusted, "trusted"),
    (Attr::Cpp, "cpp"),
    (Attr::Grader, "grader"),
    (Attr::Laws, "laws"),
    (Attr::Law, "law"),
    (Attr::Tag, "tag"),
    (Attr::Droppable, "droppable"),
    (Attr::FromOp, "from_op"),
    (Attr::MemoTable, "memo_table"),
    (Attr::Structural, "structural"),
    (Attr::Stress, "stress"),
    (Attr::Word, "word"),
    (Attr::Member, "member"),
    (Attr::Lint, "lint"),
    (Attr::OnUnimplemented, "on_unimplemented"),
    (Attr::Aka, "aka"),
    (Attr::Was, "was"),
    (Attr::SameKeys, "same_keys"),
    (Attr::Index, "index"),
    (Attr::Fast, "fast"),
    (Attr::Role, "role"),
    (Attr::Operator, "operator"),
    (Attr::Literal, "literal"),
    (Attr::Derive, "derive"),
    (Attr::Coerce, "coerce"),
    (Attr::Pattern, "pattern"),
    (Attr::Input, "input"),
];

const INTERNAL: &[(Attr, &str)] = &[(Attr::Record, "record")];

const DIRECTIVES: &[(Directive, &str)] = &[
    (Directive::Multitest, "multitest"),
    (Directive::Interactive, "interactive"),
];

const REMOVED: &[(Removed, &str)] = &[
    (Removed::Brute, "brute"),
    (Removed::Checker, "checker"),
    (Removed::Allow, "allow"),
    (Removed::Tags, "tags"),
];

fn find<T: Copy>(table: &[(T, &'static str)], name: &str) -> Option<T> {
    table.iter().find(|(_, n)| *n == name).map(|(t, _)| *t)
}

fn name_in<T: Copy + PartialEq>(table: &[(T, &'static str)], t: T) -> &'static str {
    table.iter().find(|(x, _)| *x == t).map_or("", |(_, n)| n)
}

impl Attr {
    /// The attribute `@name` is, from the decorator's name token; `@role` is one too.
    pub fn of(name: &str) -> Option<Attr> {
        find(ATTRS, name)
    }
    pub fn name(self) -> &'static str {
        match name_in(ATTRS, self) {
            "" => name_in(INTERNAL, self),
            n => n,
        }
    }
    /// Every name a source may write after `@`, for an unknown decorator's hint.
    pub fn names() -> impl Iterator<Item = &'static str> {
        ATTRS.iter().map(|(_, n)| *n)
    }
}

impl Directive {
    pub fn of(name: &str) -> Option<Directive> {
        find(DIRECTIVES, name)
    }
    pub fn name(self) -> &'static str {
        name_in(DIRECTIVES, self)
    }
}

impl Removed {
    pub fn of(name: &str) -> Option<Removed> {
        find(REMOVED, name)
    }
}

/// What a decorator sits on, for the rules of where each may go and what it takes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeclKind {
    Fn { params: usize },
    Struct,
    Enum,
    Variant,
    Field,
    Trait,
    Impl,
    Alias,
    Const,
    Domain,
    Context,
    Use,
}

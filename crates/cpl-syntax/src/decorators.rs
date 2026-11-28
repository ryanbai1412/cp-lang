//! The closed set of decorators (spec/core/06-control-functions.md, 6.4):
//! the parser rejects any other `@name`, so no later pass meets one.

/// The closed set, each decorator once: its `Mark`, its spelling and, for the live ones, what it
/// does. The parser reads a decorator's `Mark` here, once; later passes match the `Mark`.
macro_rules! marks {
    (
        [$($v:ident = $name:literal, $doc:literal;)*]
        directives: [$($dv:ident = $dname:literal;)*]
        removed: [$($rv:ident = $rname:literal;)*]
        internal: [$($iv:ident = $iname:literal;)*]
    ) => {
        /// A decorator, as the parser reads it from its text. The internal ones only the
        /// compiler makes.
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
        pub enum Mark {
            $($v,)*
            $($dv,)*
            $($rv,)*
            $($iv,)*
        }

        /// Each decorator with what it does, as completion shows it.
        pub const DECORATORS: &[(&str, &str)] = &[$(($name, $doc),)*];

        /// Old spellings the parser or checker answers with their own E0503 fix.
        pub const REMOVED: &[&str] = &[$($rname,)*];

        /// The file-level directives, also read without the `!`.
        pub const DIRECTIVES: &[&str] = &[$($dname,)*];

        impl Mark {
            const NAMES: &'static [(Mark, &'static str)] = &[
                $((Mark::$v, $name),)*
                $((Mark::$dv, $dname),)*
                $((Mark::$rv, $rname),)*
                $((Mark::$iv, $iname),)*
            ];

            /// The decorator spelled `@name`, if one is.
            pub fn of(name: &str) -> Option<Mark> {
                Self::NAMES.iter().find(|(_, n)| *n == name).map(|(m, _)| *m)
            }

            /// How it is spelled.
            pub fn name(self) -> &'static str {
                Self::NAMES.iter().find(|(m, _)| *m == self).map_or("", |(_, n)| n)
            }
        }
    };
}

marks! {
    [
    Test = "test", "tool role: a test fn `cpl test` runs";
    Alt = "alt", "tool role: `@alt(f, caps)`, another implementation of `f`";
    Gen = "gen", "tool role: a stress generator";
    Check = "check", "tool role: the checker of an output";
    Interactor = "interactor", "tool role: the judge side of an interactive problem";
    Memo = "memo", "`@memo(array)` / `@memo(map)`: cache a function";
    Get = "get", "an O(1) read-only method called without parentheses";
    Cache = "cache", "on a struct field: interior mutability";
    Bits = "bits", "on a `List<U64>` field: one bit per domain position";
    Trusted = "trusted", "`@trusted:` block: the unsafe tier";
    Cpp = "cpp", "an item written in C++: the unsafe tier";
    Grader = "grader", "`@grader(..)`: a judge entry point";
    Laws = "laws", "`@laws(sample: n)`: stress sampling of a type's laws";
    Law = "law", "`@law name(args) = condition`: a law of a type or a lent place";
    Tag = "tag", "`@tag(lit)` on a variant: the token it reads and prints as";
    Droppable = "droppable", "the result may be dropped without the unused-result lint";
    FromOp = "from_op", "on a struct's static fn: a bare op passed where the struct is wanted goes through it";
    MemoTable = "memo_table", "`@memo_table(word)` on a struct: the table a `@memo(word)` fn keeps";
    Structural = "structural", "on a trait: a tuple, record, optional, array or struct has it when each part does";
    Stress = "stress", "`@stress(kind)` on a type alias: the shape `cpl stress` draws for it";
    Word = "word", "`@word(name)` on a std struct: the lowercase call that builds it";
    Member = "member", "`@member(name)` on a std struct: the domain member (`D.name`) whose positions it is";
    Lint = "lint", "`@lint(name, from:, to:)` on a `pub fn`: a library's own lint";
    OnUnimplemented = "on_unimplemented", "on a trait: the message when a type lacks its impl";
    Aka = "aka", "`@aka(name, ..)` on a fn: other languages' names for it, for their fix";
    Was = "was", "`@was(Old, ..)`: the names a declaration replaced, for their fix";
    SameKeys = "same_keys", "on a fn: its arguments that are arrays over domains share one domain (R04b.1.3)";
    Index = "index", "on a fn: what `s[r]` calls when `r` is not a key of `s` (R07.4.17)";
    Fast = "fast", "`@fast(op)` on a fn with a body: the operation the engines and C++ run for a call";
    Role = "role", "on a declaration: the language role it fills (R11.6.1)";
    Operator = "operator", "`@operator(\"op\")` on a trait: the trait an operator calls";
    Literal = "literal", "`@literal(\"[]\")` on a trait: the trait a list literal builds through; `\"[;]\"` an array fill";
    Derive = "derive", "on a trait: every struct whose fields all have it gets it field by field";
    Coerce = "coerce", "on a fn: an implicit conversion where its result type is wanted";
    Pattern = "pattern", "on a fn: what a pattern over its type calls";
    Input = "input", "on a fn: a value it gives is input, so a range it fills is a contract";
    ]
    directives: [
    Multitest = "multitest";
    Interactive = "interactive";
    ]
    removed: [
    Brute = "brute";
    Checker = "checker";
    Allow = "allow";
    Tags = "tags";
    ]
    internal: [
    Record = "record";
    ]
}

/// Whether `@name` is a decorator, live or removed with its own fix.
pub fn is_known(name: &str) -> bool {
    DECORATORS.iter().any(|(d, _)| *d == name)
        || REMOVED.contains(&name)
        || DIRECTIVES.contains(&name)
}

/// The marker on the struct whose values are a domain's cuts (`D.cuts`).
pub const CUTS: &str = "cuts";

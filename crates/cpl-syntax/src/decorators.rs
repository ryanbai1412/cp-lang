//! The closed set of decorators (spec/core/06-control-functions.md, 6.4):
//! the parser rejects any other `@name`, so no later pass meets one.

/// Each decorator with what it does, as completion shows it.
pub const DECORATORS: &[(&str, &str)] = &[
    ("test", "tool role: a test fn `cpl test` runs"),
    (
        "alt",
        "tool role: `@alt(f, caps)`, another implementation of `f`",
    ),
    ("gen", "tool role: a stress generator"),
    ("check", "tool role: the checker of an output"),
    (
        "interactor",
        "tool role: the judge side of an interactive problem",
    ),
    ("memo", "`@memo(array)` / `@memo(map)`: cache a function"),
    ("get", "an O(1) read-only method called without parentheses"),
    ("cache", "on a struct field: interior mutability"),
    (
        "bits",
        "on a `List<U64>` field: one bit per domain position",
    ),
    ("trusted", "`@trusted:` block: the unsafe tier"),
    ("cpp", "an item written in C++: the unsafe tier"),
    ("grader", "`@grader(..)`: a judge entry point"),
    (
        "laws",
        "`@laws(sample: n)`: stress sampling of a type's laws",
    ),
    (
        "law",
        "`@law name(args) = condition`: a law of a type or a lent place",
    ),
    (
        "tag",
        "`@tag(lit)` on a variant: the token it reads and prints as",
    ),
    (
        "droppable",
        "the result may be dropped without the unused-result lint",
    ),
    (
        "from_op",
        "on a struct's static fn: a bare op passed where the struct is wanted goes through it",
    ),
    (
        "memo_table",
        "`@memo_table(word)` on a struct: the table a `@memo(word)` fn keeps",
    ),
    (
        "structural",
        "on a trait: a tuple, record, optional, array or struct has it when each part does",
    ),
    (
        "stress",
        "`@stress(kind)` on a type alias: the shape `cpl stress` draws for it",
    ),
    (
        "word",
        "`@word(name)` on a std struct: the lowercase call that builds it",
    ),
    (
        "member",
        "`@member(name)` on a std struct: the domain member (`D.name`) whose positions it is",
    ),
    (
        "lint",
        "`@lint(name, from:, to:)` on a `pub fn`: a library's own lint",
    ),
    (
        "on_unimplemented",
        "on a trait: the message when a type lacks its impl",
    ),
    (
        "aka",
        "`@aka(name, ..)` on a fn: other languages' names for it, for their fix",
    ),
    (
        "was",
        "`@was(Old, ..)`: the names a declaration replaced, for their fix",
    ),
    (
        "same_keys",
        "on a fn: its arguments that are arrays over domains share one domain (R04b.1.3)",
    ),
    (
        "index",
        "on a fn: what `s[r]` calls when `r` is not a key of `s` (R07.4.17)",
    ),
    (
        "recipe",
        "on a library struct: the value of a `-> yields T` call made outside a `for` (R07.6.2)",
    ),
    ("tuple", "on a struct: the struct an n-tuple of its field count is"),
    ("cuts", "on a struct: the cuts of a domain, `D.cuts`"),
    ("positions", "on a trait: the trait a domain's positions implement"),
    ("convert", "on a trait: the conversion `T(x)` calls"),
    (
        "operator",
        "`@operator(\"op\")` on a trait: the trait an operator calls",
    ),
    (
        "derive",
        "on a trait: every struct whose fields all have it gets it field by field",
    ),
    ("coerce", "on a fn: an implicit conversion where its result type is wanted"),
    ("pattern", "on a fn: what a pattern over its type calls"),
    (
        "variant_check",
        "on a fn: the check an `Int` converted to an enum goes through",
    ),
];

/// Old spellings the parser or checker answers with their own E0503 fix.
pub const REMOVED: &[&str] = &["brute", "checker", "allow", "tags"];

/// The file-level directives, also read without the `!`.
pub const DIRECTIVES: &[&str] = &["multitest", "interactive"];

/// Whether `@name` is a decorator, live or removed with its own fix.
pub fn is_known(name: &str) -> bool {
    DECORATORS.iter().any(|(d, _)| *d == name)
        || REMOVED.contains(&name)
        || DIRECTIVES.contains(&name)
}

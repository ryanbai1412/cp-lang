//! Syntax this version does not implement: forms the 0.3 design specifies,
//! and forms programs reach for (port dreams, other languages) that no
//! design section covers yet.
//!
//! The parser reads each form, so the statement and the names it declares
//! survive, and reports it once as "not implemented yet: <feature> (see
//! <section>)", or "(not in the design)", with what to write today. When a
//! form is implemented, its entry goes: the test that each `example` is
//! reported fails until then.

pub struct Planned {
    pub feature: &'static str,
    /// The section of the design that specifies it, as `<file> §<section>`.
    pub see: Option<&'static str>,
    /// What to write in this version instead.
    pub now: &'static str,
    /// A program whose one error is this one.
    pub example: &'static str,
}

pub const BOUNDED_STR: Planned = Planned {
    feature: "a string type with a bounded length, `Str(len: …)`",
    see: Some("design/spec/29-strings.md §29.1"),
    now: "read a `Str` and check its `.len` where it is used",
    example: "let s: Str(len: 1..=10) = read()\nprint(s.len)\n",
};

pub const FRESH_TYPE: Planned = Planned {
    feature: "a fresh type made from values, `type Z = Mint(mod: p)`",
    see: Some("design/spec/12-scoped-types.md §12.1"),
    now: "give the value as a type argument: `Mod<1000000007>`",
    example: "type Z = Mint(mod: 998244353)\nlet x: Z = 3\nprint(x)\n",
};

pub const WRAP_LINE: Planned = Planned {
    feature: "a wrap line, `--- with f() ---`",
    see: Some("design/spec/16-state.md §16.3"),
    now: "pass the rest as a closure, `f(() => …)`; for multitest/interactive use `@!multitest` or `@!interactive`",
    example: "fn go(f: () -> Int) -> Int = f()
--- with go() ---
print(1)
",
};

pub const VALUE_DECORATOR: Planned = Planned {
    feature: "a decorator after a value, `e @name(…)`",
    see: None,
    now: "write the value alone",
    example: "var dp: [0..=3]Int = 0 @rolling(1)
print(dp[0])
",
};

pub const READ_LENGTH: Planned = Planned {
    feature: "an array type whose length is read, `[read(…)]T`",
    see: None,
    now: "declare the index space first: `domain N = read(…)`, then `[N]T`",
    example: "let a: [read(1..=5)]Int = read()
print(a.len)
",
};

pub const CONTEXT_LINE: Planned = Planned {
    feature: "a context, `with io(base: 1)`",
    see: Some("design/spec/17-contexts.md §17.1"),
    now: "say it where it applies: `read<1..=n>()` for 1-based input, `print(x, precision: k)`",
    example: "with io(base: 1)
let n: 1..=5 = read()
print(n)
",
};

pub const OPERATOR_FN: Planned = Planned {
    feature: "an operator written as a fn, `fn (a: T) + (b: T) -> T`",
    see: None,
    now: "implement its trait: `impl Add for T:` with `fn add(a: T, b: T) -> T`",
    example: "struct P(x: Int)
fn (a: P) + (b: P) -> P = P(a.x + b.x)
print(1)
",
};

pub const STR_OF: Planned = Planned {
    feature: "a string type over given bytes, `Str of \"01?\"`",
    see: None,
    now: "read a `Str` and check its bytes where it is used",
    example: "let s: Str of \"01\" = read()
print(s.len)
",
};

pub const LOWER_VARIANT_PATTERN: Planned = Planned {
    feature: "a lowercase variant in a pattern, `incr(l, r) =>`",
    see: Some("design/spec/11-types.md §11.2"),
    now: "name the variant with a capital, `Incr(l, r)`",
    example: "enum Node: leaf, incr(a: Int, b: Int)
let x = Node.leaf
match x:
    incr(a, b) => print(a + b)
    _ => print(0)
",
};

pub const RECORD_DECL: Planned = Planned {
    feature: "a named record, `record Side(a: Int):`",
    see: None,
    now: "declare a struct: `struct Side(a: Int):`",
    example: "record P(x: Int)
print(P(1).x)
",
};

pub const TRAILING_LAMBDA: Planned = Planned {
    feature: "a lambda after the call, `f(n) { x => x + 1 }`",
    see: None,
    now: "pass it as the last argument: `f(n, x => x + 1)`",
    example: "fn twice(n: Int, f: (Int) -> Int) -> Int = f(f(n))
print(twice(1) { x => x + 1 })
",
};

pub const PLANNED: &[&Planned] = &[
    &BOUNDED_STR,
    &FRESH_TYPE,
    &WRAP_LINE,
    &VALUE_DECORATOR,
    &READ_LENGTH,
    &CONTEXT_LINE,
    &OPERATOR_FN,
    &STR_OF,
    &LOWER_VARIANT_PATTERN,
    &RECORD_DECL,
    &TRAILING_LAMBDA,
];

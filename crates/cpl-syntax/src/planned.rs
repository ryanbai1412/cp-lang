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

pub const CONTINUED_LINE: Planned = Planned {
    feature: "a line continued by an indented line that starts with an operator",
    see: Some("design/spec/10-syntax.md §10.1"),
    now: "wrap the expression in parentheses, inside which it can span lines",
    example: "let a = 1\n    + 2\nprint(a)\n",
};

pub const VALUE_BLOCK: Planned = Planned {
    feature: "a value on the indented lines after `=`",
    see: None,
    now: "start the value on the `=` line, as in `let x = if c:`",
    example: "let a =\n    1\nprint(a)\n",
};

pub const PAREN_SEQUENCE: Planned = Planned {
    feature: "statements in parentheses, `(a; b)`",
    see: None,
    now: "put the statements in a block, or on their own lines under an `if`",
    example: "let a = 1\nprint((print(a); a))\n",
};

pub const START_EXCLUSIVE_RANGE: Planned = Planned {
    feature: "a range that leaves out its start, `lo <.. hi`",
    see: None,
    now: "start it one later: `lo + 1 .. hi`",
    example: "for i in 0 <.. 3:\n    print(i)\n",
};

pub const DOMAIN_PARAM: Planned = Planned {
    feature: "a domain parameter, `fn f<domain D>`",
    see: None,
    now: "take it as a type parameter: `fn f<D>`",
    example: "fn f<domain D>(x: Int) -> Int = x\nprint(f(1))\n",
};

pub const DOMAIN_LIST: Planned = Planned {
    feature: "several domains on one line, `domain A = …, B = …`",
    see: None,
    now: "declare each domain on its own `domain` line",
    example: "domain A = 0..3, B = 0..4\nprint(1)\n",
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

pub const VAR_FIELD: Planned = Planned {
    feature: "a `var` field, `var steps: List<Int> = []`",
    see: None,
    now: "drop `var`: a field changes wherever its struct is a `var`",
    example: "struct S:
    var n: Int = 0
print(1)
",
};

pub const FIELD_BY_DEFAULT: Planned = Planned {
    feature: "a field typed by its default, `total = 0`",
    see: None,
    now: "give the field its type: `total: Int = 0`",
    example: "struct S:
    total = 0
print(1)
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

impl Planned {
    pub fn message(&self) -> String {
        match self.see {
            Some(see) => format!("not implemented yet: {} (see {see})", self.feature),
            None => format!("not implemented yet: {} (not in the design)", self.feature),
        }
    }
}

pub const DOMAIN_VALUE: Planned = Planned {
    feature: "a domain used as a value, `f(D)`",
    see: Some("design/spec/21-index-spaces.md §21.5"),
    now: "loop over it with `for v in D`, or pass its range, `D.first ..= D.last`",
    example: "domain D = 0..3\nfn f(n: Int) -> Int = n\nprint(f(D))\n",
};

pub const CUT_DOMAIN: Planned = Planned {
    feature: "the cuts of a domain spelled `D.Cut`",
    see: Some("design/spec/21-index-spaces.md §21.4"),
    now: "the cuts of `D` are `D.bounds` today",
    example: "domain P = 0..3\nvar a: [P.Cut]Int = 0\nprint(a[P.bounds.first])\n",
};

pub const OPERATOR_VALUE: Planned = Planned {
    feature: "an operator used as a value, `compose: +`",
    see: Some("design/stdlib/ranges.md §6.1"),
    now: "write the operation as a lambda, `(a, b) => a + b`",
    example: "let ops = (count: +, n: 2)\nprint(ops.n)\n",
};

pub const VAR_PATTERN: Planned = Planned {
    feature: "a `var` binding inside a pattern, `(var a, b)`",
    see: None,
    now: "bind the name with `let`, then copy it into a `var`",
    example: "for (var a, b) in [(1, 2)]:\n    a += b\n    print(a)\n",
};

pub const PLANNED: &[&Planned] = &[
    &BOUNDED_STR,
    &FRESH_TYPE,
    &WRAP_LINE,
    &VALUE_DECORATOR,
    &READ_LENGTH,
    &CONTEXT_LINE,
    &STR_OF,
    &LOWER_VARIANT_PATTERN,
    &VAR_FIELD,
    &FIELD_BY_DEFAULT,
    &RECORD_DECL,
    &TRAILING_LAMBDA,
    &CONTINUED_LINE,
    &VALUE_BLOCK,
    &PAREN_SEQUENCE,
    &START_EXCLUSIVE_RANGE,
    &DOMAIN_PARAM,
    &DOMAIN_LIST,
    &DOMAIN_VALUE,
    &CUT_DOMAIN,
    &OPERATOR_VALUE,
    &VAR_PATTERN,
];

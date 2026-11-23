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
    see: Some("spec/core/02-values-types.md §2.3"),
    now: "read a `Str` and check its `.len` where it is used",
    example: "let s: Str(len: 1..=10) = read()\nprint(s.len)\n",
};

pub const FRESH_TYPE: Planned = Planned {
    feature: "a fresh type made from values, `type Z = Mint(mod: p)`",
    see: Some("spec/core/02-values-types.md §2.9"),
    now: "give the value as a type argument: `Mod<1000000007>`",
    example: "type Z = Mint(mod: 998244353)\nlet x: Z = 3\nprint(x)\n",
};

pub const WRAP_LINE: Planned = Planned {
    feature: "a wrap line, `--- with f() ---`",
    see: Some("spec/core/09-contexts-checks.md §9.1"),
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

pub const CONTINUED_LINE: Planned = Planned {
    feature: "a line continued by an indented line that starts with an operator",
    see: Some("spec/core/01-lexical-grammar.md §1.2"),
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

pub const LOWER_VARIANT_PATTERN: Planned = Planned {
    feature: "a lowercase variant in a pattern, `incr(l, r) =>`",
    see: Some("spec/core/02-values-types.md §2.7"),
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

pub const INLINE_MATCH: Planned = Planned {
    feature: "`match` arms on the `match` line, `match x: 0 => a; _ => b`",
    see: None,
    now: "put each `pattern => value` arm on its own indented line",
    example: "let x = 1\nprint(match x: 0 => 5; _ => 6)\n",
};

pub const LET_IN: Planned = Planned {
    feature: "a `let … in` expression, `let y = x in y + 1`",
    see: None,
    now: "give the fn a block body, with `let y = x` on its own line",
    example: "fn f(x: Int) -> Int = let y = x - 1 in y * 2\nprint(f(3))\n",
};

pub const BRACE_SET: Planned = Planned {
    feature: "a set written in braces, `{x for x in xs}`",
    see: None,
    now: "build a list, `[x for x in xs]`, or fill a `SortedSet`",
    example: "let s = {x * 2 for x in 0..3}\nprint(s)\n",
};

pub const MAP_DEFAULT: Planned = Planned {
    feature: "a default value after a map, `m default 0`",
    see: None,
    now: "give the default where a key is looked up, with `??`",
    example: "var m: Map<Int, Int> = [0: 1] default 0\nprint(m)\n",
};

pub const LAZY_ARRAY: Planned = Planned {
    feature: "a lazily computed array, `lazy[D] i => …`",
    see: None,
    now: "build it eagerly, `[D] i => …`, or write a memoized fn",
    example: "domain D = 0..3\nlet a = lazy[D] i => i.ord * 2\nprint(a[D(1)])\n",
};

pub const FILTERED_DOMAIN: Planned = Planned {
    feature: "a domain filtered from another, `domain A = V if p(v)`",
    see: None,
    now: "keep the chosen values in a list, `[v for v in V if p(v)]`",
    example: "domain V = 0..4\ndomain A = V if v.ord > 1\nfor a in A:\n    print(a.ord)\n",
};

pub const PARAM_PATTERN: Planned = Planned {
    feature: "a pattern in place of a parameter name, `fn f((r, c): Pt)`",
    see: Some("spec/core/06-control-functions.md §6.1"),
    now: "name the parameter and take it apart on the first line, `let (r, c) = p`",
    example: "fn f((a, b): (Int, Int)) -> Int = a + b\nprint(f((1, 2)))\n",
};

pub const SOME_PATTERN: Planned = Planned {
    feature: "a `some x` pattern",
    see: None,
    now: "bind the optional, then unwrap it with `if let x = o:`",
    example: "for (x, some y) in [(1, 2)]:\n    print(x + y)\n",
};

pub const ANY_LENGTH: Planned = Planned {
    feature: "an array type of any length, `[_]T`",
    see: None,
    now: "take a `List<T>`, or name the index space, `[n]T`",
    example: "fn f(a: [_]Int) -> Int = 0\nprint(f([1, 2]))\n",
};

pub const THEN_CHAIN: Planned = Planned {
    feature: "a tie-breaker chain of comparisons, `a <=> b then c <=> d`",
    see: None,
    now: "compare tuples, `(a, c) <=> (b, d)`, or sort by a key tuple",
    example: "let a = [3, 1]\nprint(a.sort(cmp: (u, v) => u <=> v then v <=> u))\n",
};

pub const IMPLICIT_MEMBER: Planned = Planned {
    feature: "a member of the expected type without its type, `.empty`",
    see: None,
    now: "name the type, `MaxSub.empty`",
    example: "let f = _ => .empty\nprint(1)\n",
};

impl Planned {
    /// Settled as not coming in this version: the design keeps it, `plan/OPERATING.md` defers it.
    pub fn deferred(&self) -> bool {
        DEFERRED.iter().any(|d| d.feature == self.feature)
    }

    pub fn message(&self) -> String {
        match self.see {
            Some(see) if self.deferred() => format!(
                "not implemented yet, and deferred: {} (see {see})",
                self.feature
            ),
            Some(see) => format!("not implemented yet: {} (see {see})", self.feature),
            None => format!("not implemented yet: {} (not in the design)", self.feature),
        }
    }
}

pub const DOMAIN_VALUE: Planned = Planned {
    feature: "a domain used as a value, `f(D)`",
    see: Some("spec/core/02-values-types.md §2.9"),
    now: "loop over it with `for v in D`, or pass its range, `D.first ..= D.last`",
    example: "domain D = 0..3\nfn f(n: Int) -> Int = n\nprint(f(D))\n",
};

pub const CUT_DOMAIN: Planned = Planned {
    feature: "the cuts of a domain spelled `D.Cut`",
    see: Some("spec/core/04-indices.md §Domains"),
    now: "the cuts of `D` are `D.bounds` today",
    example: "domain P = 0..3\nvar a: [P.Cut]Int = 0\nprint(a[P.bounds.first])\n",
};

pub const OPERATOR_VALUE: Planned = Planned {
    feature: "an operator used as a value, `compose: +`",
    see: Some("spec/lib/ops-ranges.md §1."),
    now: "write the operation as a lambda, `(a, b) => a + b`",
    example: "let ops = (count: +, n: 2)\nprint(ops.n)\n",
};

pub const PLANNED: &[&Planned] = &[
    &BOUNDED_STR,
    &FRESH_TYPE,
    &WRAP_LINE,
    &VALUE_DECORATOR,
    &READ_LENGTH,
    &LOWER_VARIANT_PATTERN,
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
    &INLINE_MATCH,
    &LET_IN,
    &BRACE_SET,
    &MAP_DEFAULT,
    &LAZY_ARRAY,
    &FILTERED_DOMAIN,
    &PARAM_PATTERN,
    &SOME_PATTERN,
    &ANY_LENGTH,
    &THEN_CHAIN,
    &IMPLICIT_MEMBER,
];

pub const DEFERRED: &[&Planned] = &[];

# Core syntax in the spec that the checker does not accept yet

Each row is decided. Until the front end accepts it, a dream using it fails
`scripts/spec-check.sh` with the error shown, and that failure is counted as
"syntax", not as a spec gap. The syntax lane works this list top down,
front end (parse and type check) first.

## Decided, not yet accepted

| Form | Example | Decision | Today |
|---|---|---|---|
| Loop `else` | `for d in ds: ... else: print(-1)` | 125 | E0107 |
| Operators as fns | `fn (a: P) + (b: P) -> P` | 133 | E0504 |
| Recursive `let` lambdas | `let f = (n: Int) -> Int => ... f(n - 1)` | 127 | E0107 on `->` |
| Cut literals and arithmetic | `var mex: V.bounds = 0` | 124 | E0301 |
| Cut range as a piece | `cost(j..i)` where `Range<V>` is expected | 124 | E0302 |
| Chain errors | `1 < x > 2`, `a == b < c` are errors | 130 | accepted |
| Moved-from use is an error | `let b = a; a.len` | 128 | accepted |
| Overlapping `mut` places | `f(mut a[i], mut a[j])` | 128 | accepted |
| Block contexts | `with io(base: 1): ...` (block form of chapter 17) | 17.1 | E0504 |
| Field defaults | `struct S: steps: List<Int> = [], total = 0` | 134 | E0504 |
| Alphabet strings | `read<Str of "01?">()`; its chars form a domain | 134 | E0504 |
| Bounded strings | `Str(len: 1..=n)` | 134 | E0504 |

## Rejected (a dream using one is fixed instead)

| Form | Instead | Why |
|---|---|---|
| `var` on a field | plain field; mutability follows the binding | one mutability rule |
| `record Side(a: Int):` | `struct Side: a: Int` | one spelling for records |
| lowercase variant in a pattern | capitalised variants | patterns stay unambiguous |
| trailing lambda `f(n) { x => ... }` | `f(n, x => ...)`, block lambdas (98) | no braces in an indentation language |
| `[read(...)]T` | `domain V = read(...)`, then `[V]T` | the domain names the index space |
| `type Z = Mint(mod: p)` | `Mod<p>` with runtime `p` | one modular type |
| `e @name(...)` | a call | decorators sit on declarations |

## Deferred

File-wide program shape (`--- with multitest() ---`), `#set`, and testing
roles, by Ryan's call. `@!multitest` stays.

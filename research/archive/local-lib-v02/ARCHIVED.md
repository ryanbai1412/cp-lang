# Archived: the v0.2 local library

Archived unported when v0.2 was deleted (decision: Ryan, 2026-11-22, "i'm
fine to just not port examples/local-lib for now"). It is v0.2 syntax and
no gate reads it. `README.md` beside this file is the original.

Every failure under v0.3 was checked against the spec. Real gaps became
failing conformance cases; the rest is old API with a v0.3 spelling.

## Real v0.3 gaps (failing conformance cases)

| Symptom in local-lib | Case |
|---|---|
| `Fixed<D>`: `10 ** (D - k)` asks for `D: Sub` | `spec/conformance/core-08/const-param-arithmetic.cp` |
| `Range.start`, domain methods, explicit type args | already covered by `spec/conformance/` matrix cases (`index × *`, `generic × *`) |

## Old v0.2 API (no case: v0.3 spells it differently)

| v0.2 | v0.3 |
|---|---|
| `any(xs)`, `all(xs)` over Bools | `xs.any(p)`, `xs.all(p)` |
| `min(xs)`, `max(xs)` | `xs.min()`, `xs.max()` (optional) |
| `count(b for x in xs)` counting trues | `xs.count(x => b)`; `count(gen)` counts every element. Queued as the `count_bool_seq` lint |
| `r.start`, `V.K` | the position and cut API of `spec/core/04-indices.md` |
| `import std.spec` | removed module |
| `Rooted(adj: ...)` | `Rooted.new(next, root)` |
| `Fenwick.sum`, `fact.inv` property | `.prefix(cut)` / `.fold(range)`, `.inv()` method |
| `fn f(...) lends p -> T` | `fn f(...) -> T lends p` |
| `floor_sum`, `sos` runtime answers | written for C `%` and `^` as power; v0.3 has Euclidean `%`, `^` xor, `**` power |

## Removed with it

- `crates/cpl-cli/tests/pending/` (313 v0.2 programs). Main had already
  ported or removed them; this branch held a stale copy.
- 17 Rust tests are `#[ignore]` with a reason naming the case or gap;
  the 8 that name a conformance case are tracked there. The debt ratchet
  holds the count at 19 so it can only fall.

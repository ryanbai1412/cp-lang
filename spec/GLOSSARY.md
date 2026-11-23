# Glossary

One term per concept, used the same way in the spec, the stubs, the
errors, the trace and the docs. `scripts/spec-vocab.py` fails on the
spellings in the right column.

| Term | Meaning | Never |
|---|---|---|
| domain | a finite index space: declared (`domain V = 1..=n`), an integer range, or a product/compressed/pair domain | index type, index space type |
| position | an element of a domain, `V` | index (as a type name), node id |
| cut | a gap between positions, `V.cuts`; `0..=count` many | boundary, fencepost |
| piece | the positions between two cuts, a `Range<V>` | segment (except in segment trees), interval (except the `interval` module) |
| key | the domain value a sequence is indexed by, `s.keys` | offset (for a key) |
| `None`, `Some(x)` | the variants of `?T` | `none`, `some`, `null`, `nil` |
| owned / lent | the two modes of every value; parameters are lent by default | borrowed, reference (as a mode) |
| `mut` | a lent parameter or binding that may be written or drained | `&mut`, `inout` |
| `-> lends T` | a return that lends from a parameter | `lends T` without the arrow |
| operation value | a value that is a binary operation with laws (`+`, `max`, `Affine.compose`) | functor, op object |
| `law` | a property an operation value promises; stress samples it | axiom, contract (for a law) |
| `Ordering` | the result of `cmp`: `Less`, `Equal`, `Greater` | `Order` |
| `Mod<P>` | the modular integer type | `Mint`, `ModInt` |
| getter | an O(1) member read without parentheses, declared `@get pub fn` | property; `pub @get fn` |
| checked / release build | the two build modes of chapter 9 | debug build (for the checked mode) |
| trap | a checked-build stop with a source-level report | panic, abort, crash |
| `todo` | a placeholder expression; reaching it traps | `unimplemented`, `...` |
| arc | one stored direction of a graph edge in CSR | half-edge, adjacency entry |
| edge | an input edge, with its `e` id | arc (for input edges) |

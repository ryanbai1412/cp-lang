# The standard library stubs

Every public module of the standard library, as the spec wants it: signatures, doc comments and
costs. Bodies come from `stdlib/` where one matches (`scripts/port-bodies.py`); the rest are
`todo` and count as unimplemented. The library chapters in
`spec/lib/` give the model and the examples; these files are the contract an implementation is
held to. A file here shadows `stdlib/` when `CPL_STD=spec/std`.

## What every public fn documents

Two people implementing the same stub must build the same function. A doc comment starts with a
summary in prose, then a blank `///` line, then labelled lines, one fact each:

```
/// The key of the least element: `[3, 1, 2].argmin()` is `Some(1)`.
///
/// cost: O(n)
/// empty: `None`
/// ties: the first in key order
/// traps: never
/// example: `[3, 1, 2].argmin() == Some(1)`
/// example: `[5, 5].argmin() == Some(0)`
pub fn argmin<S: Keyed, K, T: Ord>(self: S) -> ?K = todo
```

| Label | Required | Says |
|---|---|---|
| `cost:` | always | time, and extra space when it is not O(1); per step for generators |
| `traps:` | always | every input that traps, or `never` |
| `example:` | always, one or more | an expression `==` its value, so it can become a test |
| `empty:` | when an input is a sequence, range, domain or container | the result, or the trap, on empty input |
| `ties:` | when the fn chooses among equal candidates (min, max, sort, search, first, nearest) | which one wins, and whether a sort is stable |
| `order:` | when it returns a sequence | the order of the output, and whose keys it carries |
| `see:` | optional | the neighbour to use instead (`sort` for `sorted`) |

Struct methods document the same way; a struct's own doc says its invariant and its layout.
`scripts/spec-stub.py --audit` lists every entry missing a required label; `--check` fails when
a file has more such entries than `doc-baseline.json` records, so the count only goes down.
Lower the baseline in the same commit that fixes entries (`--audit --write-baseline`).

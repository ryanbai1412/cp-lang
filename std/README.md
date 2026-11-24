# The standard library stubs

Every public module of the standard library, written in CP Lang and built into `cpl`: signatures,
doc comments, costs and bodies. The library chapters in `spec/lib/` give the model and the
examples; these files are the contract an implementation is held to.

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
| `traps:` | always | every input that traps, or `never`. A violated precondition is an `assert` in the body and reports R0401; a core trap (an index, overflow, `x!`) keeps its own code |
| `example:` | always, one or more | an expression `==` its value, so it can become a test |
| ```` ```cp ```` block | for anything that needs setup (a graph, a tree, a table) | a multi-line program; its last line is asserted unless it is a statement (`let`, `print`, ...). Lead with what the item computes (its defining equation or a brute force), then a recognisable contest example |
| `empty:` | when an input is a sequence, range, domain or container | the result, or the trap, on empty input |
| `ties:` | when the fn chooses among equal candidates (min, max, sort, search, first, nearest) | which one wins, and whether a sort is stable |
| `order:` | when it returns a sequence | the order of the output, and whose keys it carries |
| `see:` | optional | the neighbour to use instead (`sort` for `sorted`) |

Struct methods document the same way; a struct's own doc says its invariant and its layout.
`scripts/spec-stub.py --audit` lists every entry missing a required label; `--check` fails on
any, so a new `pub fn` lands with its labels.

## Where the surface is

The modules and every public item are the `std/*.cp` files themselves. Their
generated reference, one page per module with its prelude exports and
summary, is `skills/cp-lang/stdlib/README.md` (`scripts/skill-gen.py`), and
`cpl doc std/` builds it as a site. The naming rules, options as named
arguments and the cost line every item carries are in
`handbook/guidelines/std-api.md`. The design this library grew from is kept
in `research/design-history/stdlib-complete-design.md`.

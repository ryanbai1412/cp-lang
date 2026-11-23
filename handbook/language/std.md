# The standard library, for its users

| | |
|---|---|
| Kind | Language guide |
| Owner | stdlib |
| Describes | how a program reaches std: the prelude, `use`, modules, docs |

## By example

```cp
let a = [5, 1, 4, 1]
print(a.sorted())                 // prelude: no `use` needed
print(a.tally()[1])               // 2

use std.bag.{SortedBag}
var bag = SortedBag<Int>()
for x in a:
    bag.add(x)
print(bag.count(1))               // 2
print(bag.pop_first())            // Some(1)
```

## The model

- std is a set of modules, `std.seq`, `std.bag`, `std.graph` and so on, each
  one file `std/<m>.cp` written in plain CP Lang.
- The prelude (`std/prelude.cp`) is the list of names every program has
  without a `use`. Any std type a prelude fn returns is in the prelude too,
  so a value a program can get, it can also name in an annotation.
- Anything else is one `use std.<m>.{Name}` away. `cpl doc std.<m>` prints
  the module; the editor's hover shows the same text.
- A user library is written the same way and gets the same powers
  ([std-mechanisms.md](../design/std-mechanisms.md)).

## How std relates to the spec

| Where | What it says | Wins on conflict |
|---|---|---|
| `spec/lib/*.md` | the model, the examples, the normative rules for each facility | yes |
| `std/*.cp` | the exact signatures, docs, costs and bodies | no: a disagreement is a std bug |
| `spec/conformance/std-*/` | the cases both engines must pass | — |
| this handbook | how std is designed and written | — |

`scripts/conformance-sigs.py` and `scripts/conformance-prelude.py` check that
the spec's names and named arguments exist in std; their gap lists only
shrink.

## Reading a std doc

Every public fn's doc gives a one-line summary, then labelled lines:

```cp
/// The key of the least element: `[3, 1, 2].argmin()` is `Some(1)`.
///
/// cost: O(n)
/// empty: `None`
/// ties: the first in key order
/// traps: never
/// example: `[3, 1, 2].argmin() == Some(1)`
pub fn argmin<S: Keyed, K, T: Ord>(self: S) -> ?K
```

`cost:` is part of the contract. `traps:` lists every input that stops the
program. Each `example:` runs as a test. The full rules are in
[std-docs.md](../guidelines/std-docs.md).

## Changes

- 2026-11-23: first version.

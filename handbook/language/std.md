# The standard library, for its users

| | |
|---|---|
| Kind | Language guide |
| Owner | stdlib |
| Describes | how a program reaches std: the prelude, `use`, modules, docs |

## By example

```cp
let a = [5, 1, 4, 1]
print(a.sorted())    // prelude: no `use` needed
print(a.tally()[1])  // 2

use std.bag.{SortedBag}
var bag = SortedBag<Int>()
for x in a:
    bag.add(x)
print(bag.count(1))      // 2
print(bag.pop_first()!)  // 1: `pop_first` gives `?Int`, `!` unwraps it
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

<!-- signatures: std/seq.cp -->
```cp
/// The key of the least element: `[3, 1, 2].argmin()` is `Some(1)`.
///
/// cost: O(n)
/// empty: `None`
/// ties: the first in key order
/// traps: never
/// example: `[3, 1, 2].argmin() == Some(1)`
fn argmin<S: Keyed, D, T, K>(self: S, by: T -> K = x => x, ties: Ties
```

`cost:` is part of the contract. `traps:` lists every input that stops the
program. Each `example:` runs as a test. The full rules are in
[std-docs.md](../guidelines/std-docs.md).

## Why it looks this way

Each paragraph is one call the spec relies on, with what it beat. The full
record, and how to reverse each, is the numbered entry in `decisions.md`.

**One definition per job, over every sequence.** Sequence fns take
`self: [D]T` when they read by index, so one `runs` serves a `List<T>`, a
`[V]T` and a `Str`, and keeps `D` in what it returns (`?D`, `Range<D>`).
Pieces come back as ranges, not copied lists: a `List` per run made
`runs_by` 50x slower than a hand loop (decision 41). The stdlib surface
keeps names that already read the same everywhere, such as `first_ge` and
`last_le`, and `Mod<P>`, over respellings the core can't express without
overloading on argument labels (decision 26).

**The element type picks the algorithm, never a suffix.** Real-valued
searches are the integer ones on a `Range<F64>`:
`(0.0..1e9).bsearch(ok, steps: 100)`, `.peak(by: f, steps:)`,
`.valley(by: f, steps:)`. `bsearch_real` and `ternary_*_real` lost because a
type-named variant is three more names for the same idea (decision 419).
Interval DP is a table over `V.ranges`, whose keys are non-empty, so
`r.first` is a `V` and not a `?V`; a hand-built `[V, V.cuts]` table wasted
half its cells and left `l <= r` unchecked (decision 322).

**One elimination serves every field.** `Mat<T>.det`, `rank`, `solve`,
`solve_space` and `inverse` take `eps: ?T = None`; for an ordered field
(`F64`) the pivot is the largest entry of its column and `|x| <= eps` counts
as zero, and for `Mod<P>` any nonzero pivot works (decision 526). The
`det_real` family lost because it duplicated four elimination bodies under
second names, and its silent `1e-9` hid a tolerance the caller should see.

**A structure with a different invariant is a different type.**
`DualSegTree` (range apply, point get) and `DualFenwick` are their own
types rather than modes, because a mode would let you call `sum(r)` on a
difference array and get nonsense (decision 42). The slope trick is
`std.slope.Concave` and `Convex`, whose methods name the recurrence
(`max_convolve`, `add_quadratic`), not a treap with hand-written lambdas
every port rewrote (decision 81). A second algorithm for the same answer is
a second method named after it: `FlowGraph.push_relabel` beside `max_flow`,
like `dijkstra` beside `bellman_ford`. An `algo:` option lost because every
program then carried the unused solver into its C++ (decision 112).

**The obvious spelling is the fast one.** `Map<K, V>` is the lookup table;
when nothing observes its order, the C++ is a salted hash table, so
`HashMap` is only for a program that wants hashing regardless
(decision 231). `s.range(2..)` on a `SortedSet` or `SortedBag` is an O(1)
view that walks, counts and folds in O(log n + k), replacing five
bound-query methods with one (decision 103).

**Types carry what the algorithm produces.** A fn can return a fresh domain,
`-> (domain P, [P]Int)`, so `components` hands back its parts already
indexed by their own type (decision 33). A half-open range of cuts `j..i` is
also the piece between them, so `pre.sum(j..i)` and `a[j..i]` take it
directly; the two readings are the same set (decision 124).

**Stress harness pieces see only what a judge would.** A `@check` reads the
input and the output, never the solution's derived values, since trusting
them would pass the solution's own bug (decision 374). Checkers and
interactors are spelled as implemented, `@check fn ok()` with
`read_output()` and `@interactor fn judge(mut sol: Interactor)`, after the
spec showed parameter types that existed nowhere (decision 375).

## Changes

- 2026-11-23: first version.

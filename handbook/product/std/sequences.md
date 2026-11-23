# Sequences

| | |
|---|---|
| Kind | Product surface |
| Owner | stdlib |
| Use cases | [sequences-usecases.md](sequences-usecases.md) |

## The moment

Twenty minutes into a Div. 2, you have `a: [I]Int` and the idea: "take the
first position after the maximum where the prefix sum turns negative". In
C++ that's a loop with `max_element`, an index fix-up and a sentinel `-1`.
Here you write what you said:

```text
let m = a.argmax()!                      // a position in I, not an Int
let p = a.scan(+)                         // prefix folds, read at cuts
let k = a.keys.find(i => i > m and p.prefix(i.r) < 0)
print(k ?? -1)                            // "none" is ?I; the -1 is yours
```

Nothing to look up, no off-by-one in the cut, and `k` can't be used to
index a different array by accident.

## What good looks like

- **One vocabulary across shapes.** `[D]T`, `List<T>`, slices, `Str`,
  ranges and generators take the same calls: `find`, `count`, `fold`,
  `scan`, `runs`, `windows`, `max`, `sorted`.
- **Positions keep their index space.** A search over `[I]T` gives `?I`, a
  boundary gives `Cut<I>`, and absence is `?T`, never `-1` or `n`.
- **Options are named arguments.** `max(by:)`, `sorted(desc:)` and
  `argmax(ties:)`, not `_by` variants.
- **Views are free.** `a[l..r]`, `a.rev` and `windows(k)` don't copy, and
  they compose: `a.rev.items` keeps `a`'s keys.
- **Costs are visible.** Anything \(O(n)\) or allocating says so in its
  `cost:` line; `collect()`/`List(..)` is where a list is built.

## Dimensions

Use cases come from walking these combinations:

- **Operation:** search, extreme, fold, scan, group, sort, permute,
  window, combine, convert.
- **Shape:** array over a domain, list, slice, string, range, generator.
- **Answer:** value, position, cut, count, or a new sequence.
- **Empty input:** `?T`, the identity, or a trap, stated per call.
- **Laziness:** a view, a generator, or a built list.

## Design calls

These came from the catalogue walk and idioms' review:

- **One spelling for a position.** A position is a search over `a.keys`:
  `a.keys.find(i => p(a[i]))` gives `?I`. There is no `position`.
- **Ties belong to `max` and `min`.** `a.keys.max(by: i => a[i], ties:
  last)` is argmax; `argmax()` stays as its short form, with the same
  `ties:`.
- **A fold's seed is `init:`.** `a.fold(f, init: 0)`, matching the lint
  that suggests it.
- **One separator.** `print(a, sep: ", ")` prints any sequence; `join`
  takes any printable sequence and builds the same text.
- **A range fold needs a group.** A range read subtracts two prefixes, so
  `scan(min)` has prefix reads only. That should be a type error. Today it
  traps (seq/U7).
- **Costs are tested.** A use case that states a cost has a second program
  at \(n = 10^6\), where the wrong complexity times out.

## What the corpus says

Across 877 ports in `corpus/problems/*/port.cp`, the sequence calls used
most are `last` (276), `rev` (275), `fold` (133), `sorted` (109), `sort`
(82), `max` (74), `count` (69), `find` (64), `all` (60) and `items` (58).

Hand loops that recur, each a need the catalogue covers:

- `.last!` as a stack top: 201 uses in 120 ports (U21, U60).
- A `-1` sentinel for "not found": 86 in 49 ports (U2, U4).
- All pairs `for j in i + 1..`: 33 in 29 ports (U50).
- Adjacent pairs by index: 24 in 22 ports (U14).
- `n - 1 - i` to walk backwards: 21 in 17 ports (U16).
- `best = max(best, ..)` and `if x > best`: 21 in 18 ports (U3, U23).

[sequences-inventory.md](sequences-inventory.md) maps C++, Python, Rust
and the AtCoder Library onto the use cases.

## Where it hurts today

From probes run against main on 2026-11-23 (each repro is in the use-case
entry):

- `a.keys.find` gives `?I`, but `find` on an `[I]T` gives `?Int`, and
  there's no `ties:` on `max` or `argmax` (seq/U2, U3).
- In `cpl run`, slicing an array copies the whole array. So `runs` over an
  array, and every view that slices one, is \(O(n^2)\) (seq/U10, U17). The
  C++ is linear.
- `a.scan(min).fold(l..r)` traps instead of failing to compile (seq/U7).
- Seeded `fold`, `zip`, `perm_order`, `merge_sorted` and `compress` are in
  std but can't be called as methods (seq/U9, U13, U15, U52, U54, U25).
- A user `impl Ord` is ignored by `sort` and `max` (seq/U46).
- `join` on a `List<Int>` is a type error (seq/U24).

## Changes

- 2026-11-23: first version (RFC 0002 pilot).
- 2026-11-23: design calls, corpus tally and prior-art inventory; 67 use cases.

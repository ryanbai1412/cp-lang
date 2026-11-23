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

## Where it hurts today

From probes run against main on 2026-11-23 (each repro is in the use-case
entry):

- `find` on an `[I]T` gives `?Int`, not `?I`. `a.keys.find(..)` works, but
  that's a detour (seq/U2).
- `argmax` has no tie rule, so "the last maximum" or "widest" is a hand
  loop (seq/U3).
- `a.scan(min).fold(l..r)` silently returns garbage: range folds subtract,
  which is only right for a group (seq/U7). This is a P0.
- Seeded `fold`, `zip` and `perm_order` are in std but `cpl run` can't
  call them on lists or arrays (seq/U9, U15, U13).
- `join` on a `List<Int>` is a type error (seq/U24).

## Changes

- 2026-11-23: first version (RFC 0002 pilot).

# apio23p2

Score: 2/5
Dream: 3/5

Cold review of `port.cp` against the friction's `## Dream` (there is no `dream.cp`), without the
C++. The header reduces "v is a median" to `|#greater - #smaller| <= c` and then to two
inequalities between blocks. The port follows it with a named `Walk` monoid in a `SegTree`, a
compressed max tree and a sweep. The grouping, though, is a loop std already provides.

## Checklist failures
- idiomatic-code/R3: `var at: [Val]List<I> = []` filled by a loop (lines 19-21) is std
  `a.buckets()` (`[D]K -> [K]List<D>`). Unreported, and bookkeeping the language already removes.
- idiomatic-code/R2: the threshold sweep keeps `added` by hand (lines 40-46), in the friction.
- idiomatic-code/R2: the block spans thread `var (from, base)` through a loop (lines 58-63)
  where the dream has a `scan_with`. Unreported; filed in the friction now.
- idiomatic-code/R6: `Pre(Int(p) + 1)` (lines 55, 66) crosses from positions to prefixes
  through `Int`, in the friction.

## Owed rewrites
- `let at = a.buckets()`.

## Language gaps
- `take_through(key)` over a sorted list: in the friction.
- Position-to-prefix navigation: in the friction.
- A scan that carries its own state (`scan_with`): added to the friction, with a run.

## Dream notes
- idiomatic-code/R7: `max(for i in ...: <updates to latest> ... )` runs the tree updates inside
  a reduction expression (caps at 3). Keep the sweep's updates as statements and reduce only
  the candidate lengths.
- `a.positions()` is invented; std's `a.buckets()` is the same table.

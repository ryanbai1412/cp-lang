# Friction Report: btoi19p6

## A struct with a domain-array field cannot sit in `Heap(by:)`
- **Where:** `struct Sub(value: Int, team: List<P>, fixed: Int, banned: List<Bool>)`; the ban set
  wants to be `banned: [P]Bool` (or `Set<P>`), indexed by `p`, not `banned[p.ord]`.
- **What felt wrong:** `Heap<Sub>.new(by: s => desc(s.value))` with a `[P]Bool` or `Set<P>` field
  fails `cpl check` with "error[E0309]: Heap.less (std.heap) does not work on the types it was
  given here: a Sub has no order to compare by" (std/heap.cp:113), though `by:` gives the
  order. The workaround drops the domain (R6) and adds `.ord` at three sites.
- **Goal it hurts:** express, coherence.
- **Ideal code:** `struct Sub(value: Int, team: List<P>, fixed: Int, banned: Set<P>)` and
  `p not in banned`.
- **General fix:** check `Heap.less`'s `a < b` fallback only when no `by:` is given (Core; repro
  sent). Helps every best-first search whose state holds a domain array or set.
- **Severity:** medium

## No best-first enumeration combinator
- **Where:** the `repeat c - 1:` loop that pops, splits and pushes subproblems.
- **What felt wrong:** Lawler / k-best partitioning is a recurring pattern (k-th shortest
  path, k-th best assignment) and each port re-threads the heap by hand.
- **Goal it hurts:** express.
- **Ideal code:** `best_first(root, by: s => desc(s.value), split: s => ...).nth(c - 1)` (Dream below).
- **General fix:** a std generator `best_first(start, by:, split:) -> yields T` over `Heap`.
- **Severity:** low

## Dream

The ungated dream (no `dream.cp`: one must run, see `dream_unverified`):

```cp
let (n, k, c) = read<(1..=500, 1..=6, 1..=2000)>()
domain P = [n]
domain Ev = [k]
let score = read<[P, Ev]0..=1e6>()

struct Sub(team: [Ev]P, fixed: Ev.cuts, banned: Set<P>):
    @get fn value(self) -> Int = Ev.sum(e => self.team.max(by: p => score[p, e]))

fn best(keep: [..]P, banned: Set<P>) -> ?Sub:
    let team = keep.extend(for e in Ev.after(keep.end):
        P.filter(p => p not in banned and p not in team).max(by: p => score[p, e])?)
    Some(Sub(team, fixed: keep.end, banned))

let ranked = best_first(best([], {}), by: s => desc(s.value),
    split: s => for i in s.fixed..: best(s.team[..i], s.banned + {s.team[i]}))
print(ranked.nth(c - 1)!.value)
```

## Measurements

- Tokens: port 359, original 1069
- Max input (`gen.py --max`): port 0.01 s, 3328 KB, original 0.00 s, 3712 KB
- Emitted C++: 25100 bytes

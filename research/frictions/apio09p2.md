# apio09p2 friction

## The suffix argmin needs `suffix_fold` and a tuple key, not `.rev.scan(min(by:))`
- **Where:** `spec.cp`: `let soonest = suffix_fold(min, by_start.map(i => Some((iv[i].r, i))))`.
- **What felt wrong:** the spelling the spec teaches, `by_start.rev.scan(min(by: i => iv[i].r))`,
  fails twice: `min(by:)` is not an operation value yet (E0303), and `.rev.scan` is indexed by
  the reversed cuts, not "the same cuts" (R07.1.6; `core-07/prefix-scan-cuts.cp` fails). The
  workaround carries the key and the id as an optional tuple.
- **Goal it hurts:** express, coherence.
- **Ideal code:** `let soonest = by_start.rev.scan(min(by: i => iv[i].r))`.
- **General fix:** already routed: `min(by:)` as an op value (stdlib worklog, compiler worklog
  2026-11-22) and `Prefix` cut indexing (stdlib worklog, R07.1.6 request).
- **Severity:** medium.

## `Lift.of` needs its type arguments spelled out
- **Where:** `let chain = Lift<I, ()>.of(i => first_from(iv[i].r))`.
- **What felt wrong:** `Lift.of(...)`, as its own doc writes it, is E0302 (`V` resolves to
  `Int`); already in `std/GAPS.md` (tree: `Rooted` and `Lift`).
- **Goal it hurts:** express.
- **Ideal code:** `let chain = Lift.of(i => first_from(iv[i].r))`.
- **General fix:** infer `V` from the closure's parameter use (routed in GAPS.md).
- **Severity:** low.

## Measurements
- port.cp (spec.cp's algorithm) vs AC C++ (`corpus/submissions/5536701.cpp`) and the v0.2
  port: 60/60 `gen.py` cases each agree; `--max` (n = 2e5): native 0.36 s, 38 MB, AC C++ 0.29 s,
  same output.

## Dream

```cp
let n = read<1..=2e5>()
domain I = 1..=n
domain X = 1..=1e9
let iv = read<[I]Range<X>>()

let by_start = I.sorted(by: i => iv[i].l)
let soonest = by_start.rev.scan(min(by: i => iv[i].r))
fn first_from(c: Cut<X>) -> ?I = soonest[by_start.bsearch(i => iv[i].l < c).r?]
let chain = Lift(i => first_from(iv[i].r))
fn fit(gap: Range<X>) -> Int:
    let u = first_from(gap.l) ?? return 0
    if iv[u].r > gap.r:
        return 0
    1 + chain.climb(u, ok: v => iv[v].r <= gap.r).steps

var gaps = IntervalMap<X, ?Int>(init: None)
gaps.assign(X.range, fit(X.range))
var picked = List<I>()
for i in I:
    let (gap, best) = gaps.run_at(iv[i].first!)
    let best = best ?? continue
    if not gap.contains(iv[i]):
        continue
    let (left, right) = (gap.l..iv[i].l, iv[i].r..gap.r)
    let (a, b) = (fit(left), fit(right))
    if a + 1 + b == best:
        picked.push(i)
        gaps.assign(left, a)
        gaps.assign(iv[i], None)
        gaps.assign(right, b)
print(picked.len)
print(picked)
```

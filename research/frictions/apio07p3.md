# apio07p3

Ported from `corpus/submissions/5532787.cpp`. No `dream.cp`; the dream is below.

## Dream
The port is the dream since the Subset.of rewrite:
```cp
    let fear = View.of(Window.nth((x - pen) % n) for x in read<[fears]Pen>())
    let love = View.of(Window.nth((x - pen) % n) for x in read<[loves]Pen>())
...
fn slide(v: View, empty_new: Bool) -> View:
    let kept = v.shifted(-1)
    if empty_new: kept.with(Window.last!) else: kept
```

## Resolved: the fear/love loops, the hand shift, and the Subset collect gap
The two `var s = View.empty; for ...: s = s.with(...)` loops (R4) and the five-position
`if let down = w.prev` loop in `slide` (R3) are gone. Their missing piece was a way to
build a `Subset` from a sequence: `[W.nth(1), W.nth(2)].collect()` into a `Subset<W>` was
E0302 (collect is List-only). Content added `Subset<B>.of(xs)` for any `Seq<B>`
(80bde58b9, the SortedSet.of / Counter.of convention), and the port now uses it with
`shifted(-1)`. The Bounds header states the two list reads (`fear: [fears]1..=n`).
No open gaps.

## Measurements
- Tokens: port 369, original 677.
- `--max` (best of 3): port 63.9 ms / 5.1 MB, original 365 ms / 5.5 MB (0.18x). Same output.
- Emitted C++: 22,081 bytes (before the rewrite).
- After the rewrite: same output on `--max`; 0.02 s against 0.03 s for the loop form
  (g++ -O2, median of 5); `corpus.py verify` 0 findings.

# Friction Report: ioi09p2

## Dream

```cp
let by_rate = Worker.sort(by: i => ask[i].s / ask[i].q)   // exact rational division
var cheapest = SmallestK<Int>()                             // the k smallest, summed
```

## Gaps

- **`Ratio` does not mix with `Int`.** Integer `/` floors, so the exact rate is
  `Ratio.new(ask[i].s, ask[i].q)` (port line 18), and `Ratio * Int` doesn't
  type-check, so the bill comparison cross-multiplies `Int`s by hand (`bill_times`,
  lines 24-28) instead of comparing `rate[a.by] * a.points`. Fresh `cpl run`:
  `let r: Ratio = s / q` gives `error[E0302]: r is declared a Ratio, but s / q is an Int`;
  `Ratio.new(3, 2) * 2` gives `error[E0302]: expected Ratio, found Int`.
  General fix: promote `Int` to `Ratio` in mixed arithmetic and comparison; owner:
  stdlib. Severity: low.
- **The budget pop is a max-heap of Q kept by hand.** "keep the smallest Q
  whose sum times the rate fits W" is a running sum plus a `MaxHeap`.
  Severity: low.
- **The generator keeps every case affordable.** The accepted C++ always hires
  its first candidate, so a case where nobody fits W can't be compared; the
  generator sets W to at least the smallest S. Severity: low.

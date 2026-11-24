# Friction Report: ioi09p2

## Dream

```cp
let by_rate = Worker.sort(by: i => ask[i].s / ask[i].q)   // exact rational division
var cheapest = SmallestK<Int>()                             // the k smallest, summed
```

## Gaps

- **Exact rates need `Ratio.new` spelled out.** `S/Q` and `rate * points`
  each wrap integers in `Ratio.new(x, 1)`; integer-to-`Ratio` promotion would
  remove them. Severity: low.
- **The budget pop is a max-heap of Q kept by hand.** "keep the smallest Q
  whose sum times the rate fits W" is a running sum plus a `MaxHeap`.
  Severity: low.
- **The generator keeps every case affordable.** The accepted C++ always hires
  its first candidate, so a case where nobody fits W can't be compared; the
  generator sets W to at least the smallest S. Severity: low.

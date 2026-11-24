# Friction report: sdoi17r2p1

There is no dream.cp; the ideal is below. For Y <= 800 the port runs the exact sliding-window DP over `[Total]F64`; beyond that it uses the intended normal approximation. The max output differs from the C++ only within the checker's tolerance (`checker.py` accepts it).

## Dream

```cp
let at_most = (k: Int) => if dice <= 800: below[k + 1] else: normal_cdf((F64(k) + 0.5 - mean) / sd)
```

## The normal CDF is a hand Taylor series
- **Where:** lines 15-25: `normal_below` sums 120 terms of erf's series, with cut-offs at |z| > 6 and a written-out pi.
- **What felt wrong:** a standard special function is re-derived per port, with its accuracy argument in a comment. The series loses precision for large |x|, and the cut-offs are a hand bound on that.
- **Goal it hurts:** express, hard-to-get-wrong.
- **Ideal code:** `normal_cdf(z)` (or `erf`/`erfc`), lowered to C++'s `std::erfc`.
- **General fix:** expose `erf`, `erfc` and a `PI` constant in std's float functions. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 356, accepted C++ 645 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.14 s / 3 MB, accepted C++ 2.15 s / 121 MB.
- **Emitted C++:** 14313 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.

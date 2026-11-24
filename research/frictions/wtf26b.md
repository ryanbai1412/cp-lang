# Friction report: wtf26b

`dream.cp` is verified. The port was a slower copy: an O(N^5) DP that summed every next value `y >= x - 1` in an inner loop. The dream keeps a suffix sum over `y` and is O(N^4). The port is now the dream's body with the statement's `n: 2..=50` (the dream reads `1..=200`). It is clean under check and lint. It agrees with the v0.2 port on 30 stress cases and passes its 10 package cases.

## Dream

`dream.cp`.

## The suffix sum is a hand-written 3-D scan
- **Where:** `var suf = dp.clone()` and the `for y in (0..n - 1).rev: suf[y, k, cut] += suf[y + 1, k, cut]` triple loop.
- **What felt wrong:** "suffix sums along the first axis" is a std scan, written here as nested loops.
- **Goal it hurts:** express.
- **Ideal code:** `let suf = dp.suffix_sums(axis: 0)`.
- **General fix:** axis-wise prefix and suffix scans on multi-dimensional arrays. Owner: stdlib.
- **Severity:** low.

There are no other dream/port gaps.

## Measurements

- **Tokens:** port 321, accepted C++ 3711 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.25 s / 4 MB, accepted C++ 0.16 s / 4 MB, with the same output, on a loaded VM. In three back-to-back pairs the port ran 0.46, 0.40 and 0.66 s against the accepted C++'s 0.61, 0.86 and 1.88 s, so the order of those single runs is noise.
- **Emitted C++:** 16733 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.

## Tuned twin retired (2026-11-24)

The plain port is within 1.1x of the accepted C++ on `gen.py --max` on both counts (best of 9, CPU time and peak RSS, `scripts/corpus.py`'s judge builds): 0.025 s / 3.0 MB against 0.056 s / 3.1 MB (0.44x time, 0.96x memory), with the same output, so `perf.cp` is deleted (performance/R6 keeps a twin only over 1.1x).

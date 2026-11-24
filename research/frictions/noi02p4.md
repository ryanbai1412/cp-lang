# Friction report: noi02p4

## Dream

```cp
fn peaceful(m: Int) -> Bool = not men.pairs().any((a, b) => meet(a, b, m))
```

## Pairs instead of index loops

The v0.2 port walked `for i in 0..n for j in 0..i` over an unindexed `[n]Caveman`. The rewrite
reads the men into a `Man` domain, takes `men.pairs()`, and writes the meeting test as
`linear_congruence(..).is_some_and(..)` instead of an if/else around an optional.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++. The max case runs in
0.13 s. Corpus tests pass.

## Tuned twin retired (2026-11-24)

The plain port is within 1.1x of the accepted C++ on `gen.py --max` on both counts (best of 9, CPU time and peak RSS, `scripts/corpus.py`'s judge builds): 0.1 s / 1.9 MB against 0.119 s / 2.4 MB (0.84x time, 0.79x memory), with the same output, so `perf.cp` is deleted (performance/R6 keeps a twin only over 1.1x).

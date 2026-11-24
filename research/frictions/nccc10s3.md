# Friction report: nccc10s3

## Dream

```cp
let sa = SuffixArray(s)
let starts = RangeKth(sa.sa)
for (t, k) in read_queries():
    print(starts.kth(sa.rank_range(t), k - 1)? + 1 ?? -1)
```

## The v0.2 port was quadratic on repetitive text

`sa.find_all(t).collect().sorted()` sorts every occurrence per query, so `s = a^2e5` with
2e5 queries for `a` is 4e10 work. The rewrite takes the k-th smallest start of the suffix
array's rank range from a std persistent `RangeKth`. The new max case (that input) runs in
0.23 s.

## `kth` returns an option

std `WaveletMatrix.kth` returns `?Int` (`None` past the range), so the port's `k <= ranks.len`
check before `kth(..)!` is redundant; the port owes
`if let i = starts.kth(ranks, k - 1): i + 1 else: -1`. The earlier note here said `kth` traps.

## `k` bound

The statement's bound on `k` is not recorded here; the read takes the accepted C++'s `int`
range. In-loop reads are not stated in `// Bounds:` (the header check reads only top-level
reads).

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (new `gen.py`). Corpus tests
pass.

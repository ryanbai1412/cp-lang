// Source: Mock CCC '22 2 S3 (Kaguya Wants to Analyze the Academy Anthem),
//   https://dmoj.ca/problem/nccc10s3
// Problem: a lowercase string s (|s| <= 2e5) and q <= 2e5 queries (t, k), total |t| <= 2e5:
//   print the 1-based start of the k-th occurrence of t in s, left to right, or -1 if there
//   are fewer than k.
// Solution: the suffixes that start with t form one rank range of the suffix array, and the
//   k-th occurrence is the k-th smallest start in that range: a persistent `RangeKth` over
//   the suffix array answers it. O(n log n) build, O(|t| log n) per query.
// Bounds: q: 1..=2e5; k: 1..=2_147_483_647
// original: corpus/submissions/4214399.cpp
use std.persistent.{RangeKth}

let s: Str = read()
assert 1 <= s.len <= 200_000
let q: 1..=200_000 = read()
let sa = SuffixArray(s)
let starts = RangeKth(sa.sa)
for _ in 0..q:
    let (t, k): (Str, 1..=2_147_483_647) = read()
    let ranks = sa.rank_range(t)
    print(if k <= ranks.len: starts.kth(ranks, k - 1) + 1 else: -1)

// Source: Mock CCC '22 2 S3 (Kaguya Wants to Analyze the Academy Anthem),
//   https://dmoj.ca/problem/nccc10s3
// Problem: a lowercase string s (|s| <= 2e5) and q <= 2e5 queries (t, k), total |t| <= 2e5:
//   print the 1-based start of the k-th occurrence of t in s, left to right, or -1 if there
//   are fewer than k.
// Solution: the suffixes that start with t form one rank range of the suffix array.
//   `find_all(t)` binary-searches it in O(|t| log n) and yields their starts, which are sorted
//   to take the k-th: O(|t| log n + occ log occ) per query.
// original: corpus/submissions/4214399.cpp
use std.strings.{SuffixArray}

let s: Str = read()
let q: Int = read()
let sa = SuffixArray(s)

for _ in 0..q:
    let (t, k): (Str, Int) = read()
    let starts = sa.find_all(t).collect().sorted()
    print(if starts.len < k: -1 else: starts[k - 1] + 1)

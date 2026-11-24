// Source: NOI 2016 P1 (Good Partitions), https://dmoj.ca/problem/noi16p1
// limits: 1500 ms, 512 MB
// Problem: for each of T <= 10 lowercase strings S (n <= 30000), sum over all substrings the
//   number of ways to write the substring as AABB with A, B nonempty.
// Solution: with ends[i] the squares AA ending at i and starts[i] those starting at i, the
//   answer is sum ends[i] * starts[i + 1]. Every square lies in a run (l, r, p) with a half
//   that is a multiple of p: for each run and half w = p, 2p, ... add the intervals of starts
//   and of ends to difference arrays. Runs from lyndon_runs; O(n log n) per string.
// original: corpus/submissions/4797772.cpp
use std.strings.{lyndon_runs}

@!multitest
let s: Str = read()
let n = s.len
var starts = List.filled(n + 1, 0)
var ends = List.filled(n + 1, 0)
for run in lyndon_runs(s):
    var w = run.period
    while 2 * w <= run.r - run.l:
        let last = run.r - 2 * w
        starts[run.l] += 1
        starts[last + 1] -= 1
        ends[run.l + 2 * w - 1] += 1
        ends[last + 2 * w] -= 1
        w += run.period
let (begun, ended) = (scan(starts, +), scan(ends, +))
print(sum(ended.cut(i) * begun.cut(i + 1) for i in 0..n - 1))

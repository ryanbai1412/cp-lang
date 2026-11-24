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
domain Pos = 0..s.len
// starts[i] / ends[i]: the squares AA starting / ending at i
var starts = Diff<Pos, Int>()
var ends = Diff<Pos, Int>()
for run in lyndon_runs(s):
    for w in (run.period..=(run.r - run.l) / 2).step(run.period):
        starts.add(Pos(run.l)..=Pos(run.r - 2 * w), 1)
        ends.add(Pos(run.l + 2 * w - 1)..=Pos(run.r - 1), 1)
let (begun, ended) = (starts.finish(), ends.finish())
print(sum(ended[Pos(i)] * begun[Pos(i + 1)] for i in 0..s.len - 1))

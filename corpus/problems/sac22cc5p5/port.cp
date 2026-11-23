// Source: SAC '22 Code Challenge 5 P5 (Querying Rectangles), https://dmoj.ca/problem/sac22cc5p5
// Problem: an N x M grid of lowercase letters (N, M <= 5000) and 1 <= K <= min(N, M). Print the
//   number of distinct K x K subgrids.
// Solution: a 2D double polynomial hash (`Hash2D`, prefix sums) gives each K x K square's hash in
//   O(1); put all (N - K + 1)(M - K + 1) of them in a Set and print its size. O(N M) hashes,
//   collisions ignored.
// Runnable today/ subset.
// original: corpus/submissions/4575631.cpp
use std.strings.{Hash2D, HashVal}

let (n, m, k): (1..=5_000, 1..=5_000, 1..=5_000) = read()
domain Row = 0..n
domain Col = 0..m
let g: [Row, Col]Char = read()

let h = Hash2D<Row, Col>.new((r, c) => Int(g[r, c]))
var seen = Set<HashVal>()
for r in 0..=n - k:
    for c in 0..=m - k:
        seen.add(h.rect(Row(r)..=Row(r + k - 1), Col(c)..=Col(c + k - 1)))
print(seen.len)

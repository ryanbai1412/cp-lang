// Source: COCI 2025/2026 Round 3 #2 (Festival); not on DMOJ, statement in the round's task
//   PDF in the COCI 2025/2026 archive
// limits: 1000 ms, 512 MB
// Problem: put n distinct candies (n <= 5000) into k identical non-empty boxes (k <= n),
//   each box ordered with its largest candy first. Count the arrangements modulo 1e9 + 7.
//   A box is a cycle, so this is the number of permutations of n with exactly k cycles.
// Solution: unsigned Stirling numbers of the first kind, c(n, k), read off std's whole row
//   `stirling1_row(n)`.
// original: corpus/submissions/7730377.cpp
// Bounds: n: 1..=5000; k: 1..=n

use std.poly.{stirling1_row}

const P = 1000000007

let n: 1..=5000 = read()
let k: 1..=n = read()
print(stirling1_row<P>(n)[k])

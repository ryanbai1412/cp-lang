// Source: COCI 2025/2026 Round 3 #2 (Festival); not on DMOJ, statement in the round's task
//   PDF in the COCI 2025/2026 archive
// limits: 1000 ms, 512 MB
// Problem: put n distinct candies (n <= 5000) into k identical non-empty boxes (k <= n),
//   each box ordered with its largest candy first. Count the arrangements modulo 1e9 + 7.
//   A box is a cycle, so this is the number of permutations of n with exactly k cycles.
// Solution: unsigned Stirling numbers of the first kind: item i starts a cycle of its own
//   or goes right after one of the i - 1 items before it, so c(i, c) = c(i - 1, c - 1) +
//   (i - 1) c(i - 1, c). One row rolled from high c down: O(n^2).
// original: corpus/submissions/7730377.cpp
const P = 1000000007
type Z = Mod<P>

let n: 1..=5000 = read()
let k: 1..=n = read()

// ways[c]: permutations of the items so far with c cycles. Item i either
// starts a cycle of its own or goes right after one of the i - 1 before it.
var ways: [0..=n]Z = 0
ways[0] = 1
for i in 1..=n:
    for c in (1..=i).rev: ways[c] = ways[c - 1] + ways[c] * (i - 1)
    ways[0] = 0
print(ways[k])

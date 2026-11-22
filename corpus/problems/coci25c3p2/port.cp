// COCI 2025 Contest 3 P2: count the permutations of n items that have
// exactly k cycles, modulo 1e9 + 7.
// original: corpus/submissions/7730377.cpp
const P = 1000000007
type Z = Mod<P>

let n: 1..=5000 = read()
let k: 1..=5000 = read()
assert k <= n

// ways[c]: permutations of the items so far with c cycles. Item i either
// starts a cycle of its own or goes right after one of the i - 1 before it.
var ways: [0..=n]Z = 0
ways[0] = 1
for i in 1..=n:
    for c in (1..=i).rev: ways[c] = ways[c - 1] + ways[c] * (i - 1)
    ways[0] = 0
print(ways[k])

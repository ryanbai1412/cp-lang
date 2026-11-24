// Source: Canada Day Contest 2021 P5, https://dmoj.ca/problem/canada21p5
// Problem: binary strings s, t of length n <= 128. Each second x of the n bits of s (0 <= x <= n),
//   chosen uniformly, are flipped. Print the probability that s equals t after K <= 1e9
//   seconds, modulo 1e9+7. (Here x is m and K is k.)
// Solution: only the number of differing places matters: a Markov chain on 0..=n where from
//   i, c of the m flips hit differing places with probability C(i, c) C(n - i, m - c) / C(n, m).
//   Raise the (n + 1)^2 matrix to the k with std `pow` over the matrix-product monoid, O(n^3 log k). By symmetry the answer is
//   P(0 -> d) / C(n, d) for the starting distance d.
// original: corpus/submissions/3718293.cpp
const P = 1000000007
type Z = Mod<P>

let n: 1..=128 = read()
let m: 0..=n = read()
let k: 0..=1e9 = read()
let s: Str = read()
let t: Str = read()
domain Off = 0..=n  // how many places differ
let comb = Comb<Z>(n)

// One round from i differing places: c of the m flips land on them.
var step: [Off, Off]Z = Z.zero
for i in Off:
    for c in 0..=min(i.ord, m):
        if let j = Off.at(i.ord - c + (m - c)):
            step[i, j] += comb.choose(i.ord, c) * comb.choose(n - i.ord, m - c) / comb.choose(n, m)

// Walk back from "equal": every string at distance d is equally likely.
let d = (0..n).count(i => s[i] != t[i])
fn mul(a: [Off, Off]Z, b: [Off, Off]Z) -> [Off, Off]Z = [Off, Off] (i, l) => sum(a[i, j] * b[j, l] for j in Off)
let one: [Off, Off]Z = [Off, Off] (i, j) => Z(Int(i == j))
let walk = pow(step, k, op: Monoid(one, mul))
print(walk[Off.first!, Off(d)] / comb.choose(n, d))

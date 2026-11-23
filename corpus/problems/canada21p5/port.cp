// Source: Canada Day Contest 2021 P5, https://dmoj.ca/problem/canada21p5
// Problem: binary strings s, t of length n <= 128. Each second x of the n bits of s (0 <= x <= n),
//   chosen uniformly, are flipped. Print the probability that s equals t after K <= 1e9
//   seconds, modulo 1e9+7. (Here x is m and K is k.)
// Solution: only the number of differing places matters: a Markov chain on 0..=n where from
//   i, c of the m flips hit differing places with probability C(i, c) C(n - i, m - c) / C(n, m).
//   Raise the (n + 1)^2 matrix to the k by squaring, O(n^3 log k). By symmetry the answer is
//   P(0 -> d) / C(n, d) for the starting distance d.
// original: corpus/submissions/3718293.cpp

const P = 1000000007
type Z = Mod<P>

let (n, m, k): (Int, Int, Int) = read()  // m <= n <= 128, k <= 1e9
let (s, t): (Str, Str) = read()
domain Off = 0..=n  // how many places differ
type Mat = [Off, Off]Z
let comb = Comb<Mod<P>>(n)

fn mul(a: Mat, b: Mat) -> Mat:
    var c: Mat = [Off, Off] (i, j) => Z.zero
    for i in Off:
        for j in Off:
            if a[i, j] != Z.zero:
                for l in Off: c[i, l] += a[i, j] * b[j, l]
    c

fn power(x: Mat, e: Int) -> Mat:
    var (base, acc, left) = (x, [Off, Off] (i, j) => Z(Int(i == j)), e)
    while left > 0:
        if left % 2 == 1: acc = mul(acc, base)
        base = mul(base, base)
        left /= 2
    acc

// One round from i differing places: c of the m flips land on them.
var step: Mat = [Off, Off] (i, j) => Z.zero
for i in Off:
    for c in 0..=min(i.to_idx, m):
        if let j = Off.at(i.to_idx - c + (m - c)):
            step[i, j] += comb.choose(i.to_idx, c) * comb.choose(
                n - i.to_idx,
                m - c,
            ) / comb.choose(n, m)

// Walk back from "equal": every string at distance d is equally likely.
let d = sum(1 for i in 0..n if s[i] != t[i])
print(power(step, k)[Off(0), Off(d)] / comb.choose(n, d))

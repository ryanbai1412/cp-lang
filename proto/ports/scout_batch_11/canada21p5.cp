// Canada Day Contest 2021 P5: two binary strings of length n; each of k
// rounds flips exactly m random places of the first. The probability that
// it then equals the second, modulo 1e9+7.
// Only the number of places that differ matters: a Markov chain on 0..=n.
// original: corpus/submissions/3718293.cpp
use std.mod.{Mod, Comb}

const P = 1000000007
type Z = Mod<P>

let (n, m, k): (Int, Int, Int) = read()     // m <= n <= 128, k <= 1e9
let (s, t): (Str, Str) = read()
domain Off = 0..=n                           // how many places differ
type Mat = [Off, Off]Z
let comb = Comb<P>.new(n)

fn mul(a: Mat, b: Mat) -> Mat:
    var c: Mat = [Off, Off] (i, j) => Z(0)
    for i in Off:
        for j in Off:
            if a[i, j] != Z(0):
                for l in Off: c[i, l] += a[i, j] * b[j, l]
    c

fn power(x: Mat, e: Int) -> Mat:
    var (base, acc, left) = (x, [Off, Off] (i, j) => Z(if i == j: 1 else: 0), e)
    while left > 0:
        if left % 2 == 1: acc = mul(acc, base)
        base = mul(base, base)
        left /= 2
    acc

// One round from i differing places: c of the m flips land on them.
var step: Mat = [Off, Off] (i, j) => Z(0)
for i in Off:
    for c in 0..=min(i.to_idx, m):
        if let j = Off.try(i.to_idx - c + (m - c)):
            step[i, j] += comb.choose(i.to_idx, c) * comb.choose(n - i.to_idx, m - c) / comb.choose(n, m)

// Walk back from "equal": every string at distance d is equally likely.
let d = sum(1 for i in 0..n if s[i] != t[i])
print(power(step, k)[Off(0), Off(d)] / comb.choose(n, d))

// Source: NOI 2019 P5 Landlords (斗主地), https://dmoj.ca/problem/noi19p5
// limits: 1000 ms, 512 MB
// Problem: a deck of 3 <= n <= 1e7 cards, the card at (0-indexed) position t scoring t + 1
//   (type 1) or (t + 1)^2 (type 2). Each of m <= 5e5 shuffles cuts off the top A_i cards and
//   merges the two piles, taking the next card from a pile with probability proportional to its
//   remaining size. Then Q <= 5e5 queries: print the expected score of the card at position c,
//   mod 998244353.
// Solution: every interleaving of the two piles is equally likely, so the expected score stays a
//   polynomial of degree <= 2 in the position. Keep its coefficients; after a shuffle the card at
//   position t <= 2 is the top pile's j-th or the bottom pile's (t - j)-th card, where j of the
//   first t cards came from the top: weigh the old polynomial there by the falling-factorial
//   probability of that origin sequence, and interpolate the new values at 0, 1, 2. O(m + Q).
// original: corpus/submissions/5652798.cpp
// Bounds: n: 3..=1e7; m, q: 1..=5e5; kind: 1..=2; a, c: 1..=n
use std.math.{choose}
use std.poly.{interpolate}

type Z = Mod<998_244_353>

let (n, m, kind): (3..=1e7, 1..=5e5, 1..=2) = read()
let xs = [Z(t) for t in 0..3]
var f = interpolate(xs, [Z(t + 1) ** kind for t in 0..3])

fn falling(x: Int, k: Int) -> Z = prod(Z(x - i) for i in 0..k)

repeat m:
    let a: 1..=n = read()
    let b = n - a
    f = interpolate(xs, [
        sum(Z(choose(t, j)) * (falling(a, j + 1) * falling(b, t - j) * f.eval(Z(j))
            + falling(a, j) * falling(b, t - j + 1) * f.eval(Z(a + t - j))) for j in 0..=t)
            / falling(n, t + 1)
        for t in 0..3])

let q: 1..=5e5 = read()
repeat q:
    let c: 1..=n = read()
    print(f.eval(Z(c - 1)))

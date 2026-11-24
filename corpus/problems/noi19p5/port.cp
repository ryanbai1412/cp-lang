// Source: NOI 2019 P5 Landlords (斗主地), https://dmoj.ca/problem/noi19p5
// limits: 1000 ms, 512 MB
// Problem: a deck of 3 <= n <= 1e7 cards, the card at (0-indexed) position t scoring t + 1
//   (type 1) or (t + 1)^2 (type 2). Each of m <= 5e5 shuffles cuts off the top A_i cards and
//   merges the two piles, taking the next card from a pile with probability proportional to its
//   remaining size. Then Q <= 5e5 queries: print the expected score of the card at position c,
//   mod 998244353.
// Solution: every interleaving of the two piles is equally likely, so the expected score stays a
//   polynomial of degree <= 2 in the position. Keep its values at 0, 1, 2; after a shuffle the
//   card at position t <= 2 is the top pile's j-th or the bottom pile's (t - j)-th card, where j
//   of the first t cards came from the top: weigh the old polynomial there by the
//   falling-factorial probability of that origin sequence. O(m + Q).
// original: corpus/submissions/5652798.cpp
// Bounds: n: 3..=1e7; m, q: 1..=5e5; kind: 1..=2; a: 0..=n; c: 1..=n
use std.math.{choose}
type Z = Mod<998_244_353>

let (n, m, kind): (3..=1e7, 1..=5e5, 1..=2) = read()
let half = 1 / Z(2)
let inv_falling_n = [1 / falling(n, t + 1) for t in 0..3]

// The quadratic through (0, ys[0]), (1, ys[1]), (2, ys[2]), at x.
fn at(ys: List<Z>, x: Z) -> Z = (
    ys[0] * (x - 1) * (x - 2) * half - ys[1] * x * (x - 2) + ys[2] * x * (x - 1) * half
)

fn falling(x: Int, k: Int) -> Z = prod(Z(x - i) for i in 0..k)

var f = [Z(t + 1) ** kind for t in 0..3]
repeat m:
    let a: 0..=n = read()
    let b = n - a
    f = [
        sum(
            Z(choose(t, j)) * (
                falling(a, j + 1) * falling(b, t - j) * f[j] + falling(a, j) * falling(
                    b,
                    t - j + 1,
                ) * at(
                    f,
                    Z(a + t - j),
                )
            ) for j in 0..=t
        ) * inv_falling_n[t] for t in 0..3
    ]

let q: 1..=5e5 = read()
repeat q:
    let c: 1..=n = read()
    print(at(f, Z(c - 1)))

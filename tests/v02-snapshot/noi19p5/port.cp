// Source: NOI 2019 P5 Landlords (斗主地), https://dmoj.ca/problem/noi19p5
// Problem: a deck of n <= 1e7 cards, card i scoring i (type 1) or i^2 (type 2). Each of m <= 5e5
//   shuffles cuts off the top A_i cards and merges the two piles, taking the next card from a pile
//   with probability proportional to its remaining size. Then Q <= 5e5 queries: print the
//   expected score of the card at position c, mod 998244353.
// Solution: the expected score stays a polynomial of degree <= 2 in the position, since the merge
//   sends each pile's j-th card to a position distributed linearly. Keep its coefficients; after
//   each shuffle compute the new expectations at positions 1, 2 and n from the old polynomial
//   (which card can land there and with what probability) and refit. n <= 2 is special-cased.
//   O(m + Q).

type Z = Mod<998244353>

let (n, m, kind): (Int, Int, Int) = read()
var cuts: List<Int> = []
for _ in 0..m:
    let a: Int = read()
    cuts.push(a)
let q: Int = read()
var asks: List<Int> = []
for _ in 0..q:
    let c: Int = read()
    asks.push(c)

fn eval(a: Z, b: Z, c: Z, x: Int) -> Z:
    let zx = Z(x)
    (a * zx + b) * zx + c

var a: Z = if kind == 1: 0 else: 1
var b: Z = Int(kind == 1)
var c: Z = 0
if n == 1:
    for i in 0..q:
        print(1)
elif n == 2:
    let inv2 = Z(2).inv()
    for idx in 0..m:
        let cut = cuts[idx]
        if cut == 1:
            let f1 = eval(a, b, c, 1)
            let f2 = eval(a, b, c, 2)
            let avg = (f1 + f2) * inv2
            a = 0
            b = 0
            c = avg
    for i in 0..q:
        print(eval(a, b, c, asks[i]))
else:
    let inv_n = Z(n).inv()
    let inv_n1 = Z(n - 1).inv()
    let inv_2 = Z(2).inv()
    let inv_n2 = Z(n - 2).inv()

    for idx in 0..m:
        let cut = cuts[idx]
        let r = n - cut
        let p = Z(cut) * inv_n
        let q_prob = Z(r) * inv_n

        let g1 = p * eval(a, b, c, 1) + q_prob * eval(a, b, c, cut + 1)

        var g2: Z = 0
        if cut >= 2:
            g2 += p * Z(cut - 1) * inv_n1 * eval(a, b, c, 2)
        if cut >= 1 and r >= 1:
            g2 += p * Z(r) * inv_n1 * eval(a, b, c, cut + 1)
            g2 += q_prob * Z(cut) * inv_n1 * eval(a, b, c, 1)
        if r >= 2:
            g2 += q_prob * Z(r - 1) * inv_n1 * eval(a, b, c, cut + 2)

        let gn = p * eval(a, b, c, cut) + q_prob * eval(a, b, c, n)

        let d1 = g2 - g1
        let slope = (gn - g1) * inv_n1
        a = (slope - d1) * inv_n2
        b = d1 - 3 * a
        c = g1 - a - b

    for i in 0..q:
        print(eval(a, b, c, asks[i]))

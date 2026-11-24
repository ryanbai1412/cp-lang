// Source: SHOI 2016 (Random Sequence), https://dmoj.ca/problem/shoi16p5
// limits: 2000 ms, 256 MB
// Problem: n <= 1e5 numbers a_i <= 1e4 in a row; putting +, - or * between each neighbouring
//   pair gives 3^(n-1) expressions. After each of q <= 1e5 point updates a_t = v, print the
//   sum of the values of all expressions, mod 1e9+7.
// Solution: flipping a + to - negates the term after it, so every term but the first product
//   run cancels: the sum is sum over i of (a_1 * ... * a_i) * (sign choices where the first
//   run ends at i), which is 2 * 3^(n-1-i) for i < n and 1 for i = n. Each position is the
//   std Affine map x => a_i * x + a_i * that count, and a SegTree of their compositions keeps
//   the whole sum: O(log n) per update. The run counts are one table, computed once.
// original: corpus/submissions/5445695.cpp
// Bounds: n, q: 1..=1e5; a, v: 0..=1e4; t: 1..=n
use std.algebra.{Affine}

type Z = Mod<1_000_000_007>

let (n, q): (1..=1e5, 1..=1e5) = read()
domain Pos = 1..=n
var a: [Pos]0..=1e4 = read()

// a product run ending at i < n is closed by + or - and followed by any of 3^(n-1-i) choices
let closes = [Pos] i => if i.label == n: Z(1) else: 2 * Z(3) ** (n - 1 - i.label)
fn leaf(i: Pos) -> Affine<Z> = Affine(Z(a[i]), Z(a[i]) * closes[i])

// the fold nests each position's map inside the one before it
var chain = SegTree(Monoid(Affine<Z>.id(), (f, g) => Affine.then(g, f)), [Pos] i => leaf(i))
for _ in 0..q:
    let (t, v): (Pos, 0..=1e4) = read()
    a[t] = v
    chain.set(t, leaf(t))
    print(chain.all.b)

// Source: SHOI 2016 (Random Sequence), https://dmoj.ca/problem/shoi16p5
// Problem: n <= 1e5 numbers a_i <= 1e4 in a row; putting +, - or * between each neighbouring
//   pair gives 3^(n-1) expressions. After each of q <= 1e5 point updates a_t = v, print the
//   sum of the values of all expressions, mod 1e9+7.
// Solution: flipping a + to - negates the term after it, so every term but the first product
//   run cancels: the sum is sum over i of (a_1 * ... * a_i) * (sign choices where the first
//   run ends at i), which is 2 * 3^(n-1-i) for i < n and 1 for i = n. Each position is the
//   affine map x => a_i * x + a_i * that count, and a SegTree of their compositions keeps
//   the whole sum: O(log n) per update.
// original: corpus/submissions/5445695.cpp
const MOD = 1000000007
type Z = Mod<MOD>
let n: 1..=1e5 = read()
let q: 0..=1e5 = read()
domain I = 0..n
var a: [I]Int = read()

struct Affine(mul: Z, add: Z)

// position i ends a product run in 2 * 3^(n-2-i) of the sign choices after it
fn leaf(i: I) -> Affine:
    let at = i.ord
    let closes = if at == n - 1: Z(1) else: 2 * Z(3) ** (n - 2 - at)
    Affine(Z(a[i]), Z(a[i]) * closes)

let compose = Monoid<Affine>(Affine(Z(1), Z(0)), (f, g) => Affine(f.mul * g.mul, f.mul * g.add + f.add))
var chain = SegTree(compose, [I] i => leaf(i))
for _ in 0..q:
    let i: 1..=n = read()
    let x: Int = read()
    let at = I.nth(i - 1)
    a[at] = x
    chain.set(at, leaf(at))
    print(chain.all.add)

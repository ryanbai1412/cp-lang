// SHOI 2016 P5: the expected value of a random +/* expression under point updates, as a segment tree of affine maps over Mod<P>.
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
    let at = i.to_idx
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

// SHOI 2016 P5, expected-value sum of a random +/* expression under point updates
// original: corpus/submissions/5445695.cpp
type Z = Mod<1e9 + 7>
let n: 1..=1e5 = read()
let q: 0..=1e5 = read()
domain I = 1..=n
var a: [I]Int = read()

type Affine = (mul: Z, add: Z)             // x -> mul * x + add

// position i ends a product run in 2 * 3^(n-1-i) of the sign choices after it
fn leaf(i: I) -> Affine:
    let closes: Z = if i == I.last: 1 else: 2 * Z(3) ** (n - 1 - i)
    (a[i], a[i] * closes)

var chain = SegTree<I, Affine>((1, 0), (f, g) => (f.mul * g.mul, f.mul * g.add + f.add), [I] i => leaf(i))
for _ in 0..q:
    let (i, x): (I, Int) = read()
    a[i] = x
    chain.set(i, leaf(i))
    print(chain.fold(I).add)

// SHOI 2016 P5, expected-value sum of a random +/* expression under point updates (buildable today-form)
// original: corpus/submissions/5445695.cpp
// The affine maps carry plain Int fields reduced by hand: a struct or a
// tuple holding Mod<P> does not build in the prototype.
const MOD = 1000000007
let n: 1..=1e5 = read()
let q: 0..=1e5 = read()
domain I = 0..n
var a: [I]Int = read()

struct Affine:
    mul: Int
    add: Int

var pow3: List<Int> = []
pow3.push(1)
for i in 1..n: pow3.push(pow3[i - 1] * 3 % MOD)

// position i ends a product run in 2 * 3^(n-2-i) of the sign choices after it
fn leaf(i: I) -> Affine:
    let at = i.to_idx
    let closes = if at == n - 1: 1 else: 2 * pow3[n - 2 - at] % MOD
    Affine(a[i] % MOD, a[i] % MOD * closes % MOD)

var chain = SegTree<I, Affine>(
    Affine(1, 0),
    (f, g) => Affine(f.mul * g.mul % MOD, (f.mul * g.add + f.add) % MOD),
)
for i in I: chain.set(i, leaf(i))
for _ in 0..q:
    let i: 1..=n = read()
    let x: Int = read()
    let at = I.nth(i - 1)
    a[at] = x
    chain.set(at, leaf(at))
    print(chain.all.add)

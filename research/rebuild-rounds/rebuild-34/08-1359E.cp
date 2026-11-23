// CF 1359E Modular Stability: stable iff every element is a multiple of the
// smallest one, so count = sum over a1 of C(n / a1 - 1, k - 1).
type Z = Mod<998244353>
let n: 1..=5e5 = read()
let k: 1..=5e5 = read()
let c = Comb<Z>(n)
print((1..=n).map(a1 => c.choose(n / a1 - 1, k - 1)).sum())

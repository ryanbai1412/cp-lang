// Codeforces 2066 D1: Club of Young Aircraft Builders (easy version).
// Processing floors 1..n - 1 from bottom to top, each floor chooses k in 0..=c
// of its first c unseen slots in C(c, k) ways, and floor n takes the last c.
// The generating function is (1 + x)^(c * (n - 1)), so the answer is
// C(c * (n - 1), m - c).
type M = Mod<1_000_000_007>
let comb = Comb<M>(10_000)

@!multitest
let n: 1..=100 = read()
let c: 1..=100 = read()
let m: c..=n * c = read()
let _: [0..m]0..=0 = read()
print(comb.choose(c * (n - 1), m - c))

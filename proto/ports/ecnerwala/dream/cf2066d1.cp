// Codeforces 2066 D1: Club of Young Aircraft Builders (easy version) (dream).
// Processing floors 1..n - 1 from bottom to top, each floor chooses k in 0..=c
// of its first c unseen slots in C(c, k) ways, and floor n takes the last c.
// The generating function is (1 + x)^(c * (n - 1)), so the answer is
// C(c * (n - 1), m - c).
type M = Mint(mod: 1e9 + 7)
let comb = Comb<M>(1e4)

--- multitest ---
let (n, c, m): (1..=100, 1..=100, c..=n * c) = read()
skip<[m]0..=0>()
print(comb.choose(c * (n - 1), m - c))

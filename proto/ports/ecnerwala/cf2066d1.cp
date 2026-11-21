// Codeforces 2066 D1: Club of Young Aircraft Builders (easy version).
// Processing floors 1..n - 1 from bottom to top, each floor chooses k in 0..=c
// of its first c unseen slots in C(c, k) ways, and floor n takes the last c.
// The generating function is (1 + x)^(c * (n - 1)), so the answer is
// C(c * (n - 1), m - c).
let comb = Comb<1_000_000_007>(10_000)

let t: 1..=10_000 = read()
for _ in 0..t:
    let (n, c, m): (1..=100, 1..=100, 1..=10_000) = read()
    let a: [0..m]Int = read()
    assert (a.len == m)
    print(comb.choose(c * (n - 1), m - c))

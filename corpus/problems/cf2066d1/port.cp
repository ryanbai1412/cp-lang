// Source: Codeforces 2066D1 (Club of Young Aircraft Builders, easy version),
//   https://codeforces.com/contest/2066/problem/D1
// limits: 4000 ms, 1000 MB
// Problem: floors 1..=n (n <= 100) launch m planes in turn (c <= m <= n c, c <= 100); a launch
//   from floor i is seen on floors 1..=i, a resident stops once they have seen c, and every
//   resident must see >= c. All records are lost: count the floor sequences of length m, mod
//   1e9+7. Multitest, sum m <= 1e4.
// Solution: floor 1 sees every plane, so its planes are some k of the first c slots, and removing
//   them leaves the same problem on n - 1 floors: ways(n, m) = sum C(c, k) ways(n - 1, m - k),
//   ways(1, m) = [m == c]. That is the x^(m-c) coefficient of (1 + x)^(c (n - 1)), so print
//   C(c (n - 1), m - c) from a factorial table (`Comb`), O(1) per test.
// Bounds: n, c: 1..=100; m: c..=n * c; _: 0..=0
type M = Mod<1_000_000_007>
let comb = Comb<M>(10_000)

@!multitest
let n: 1..=100 = read()
let c: 1..=100 = read()
let m: c..=n * c = read()
let _: [0..m]0..=0 = read()
print(comb.choose(c * (n - 1), m - c))

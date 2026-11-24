// Source: Codeforces 2262C (Traveling the World), https://codeforces.com/contest/2262/problem/C
// limits: 2000 ms, 256 MB
// Problem: n values 0 = a_1 < ... < a_n (4 <= n <= 2e5, a_i <= 1e9). Count the orders b of
//   them in which a path visits every position once, moving i -> j only when b_i + b_j =
//   max(b_i..b_n). Mod 1e9+7, multitest, sum of n <= 2e5.
// Solution: a good b needs the sorted values to be two progressions of one step d: 0, d, ..,
//   (half - 1)d and top, top - d, .. (half = n/2, top = a_n), so d is top - a[n-2] or
//   a[n-2] / (half - 1); check each candidate by merging. Every valid d gives
//   half! * (n - half - 2)! good orders. O(n log n).
// Bounds: n: 4..=2e5
const MOD = 1e9 + 7
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let n: 4..=2e5 = read()
let a: [0..n]Int = read()
let (top, half) = (a[n - 1], n / 2)
let ds = [top - a[n - 2], a[n - 2] / (half - 1)].sorted().deduped()
let ok = ds.count(d =>
    [d * i for i in 0..half].merge_sorted([top - d * i for i in (0..n - half).rev]) == a
)
print(comb.fact(n - half - 2) * comb.fact(half) * ok)

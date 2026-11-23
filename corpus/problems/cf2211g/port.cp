// Source: Codeforces 2211G (Rational Bubble Sort), https://codeforces.com/contest/2211/problem/G
// Problem: n <= 1e6 integers 0 <= a_i <= 1e6. An operation replaces a_i and a_(i+1) both by their
//   average. Print whether a can be made non-decreasing. Multitest, sum of n <= 1e6.
// Solution: operations keep the sum. With diff(i) = n * (sum of a[..i]) - tot * i: if some proper
//   prefix averages strictly below the mean (diff(i) < 0), the answer is Yes. Otherwise a sorted
//   result must be constant, which needs the exact-mean cuts (diff(i) = 0) to split a into
//   blocks of length <= 2, each levelled by one operation: every element is next to such a cut.
//   Exact integers on prefix sums, O(n).

@!multitest
let n: 1..=1_000_000 = read()
let a: [0..n]0..=1_000_000 = read()
let p = prefix(a)
let tot = a.sum()
fn diff(i: Int) -> Int: p.fold(..i) * n - tot * i

let below = (1..n).any(i => diff(i) < 0)
let no_gap = (0..n).all(i => diff(i) == 0 or diff(i + 1) == 0)
print(if below or no_gap: "Yes" else: "No")

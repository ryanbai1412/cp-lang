// Codeforces 2211 G — Prefix Average Balance
//
// Let `tot = a.sum()` and `diff(i) = p.sum(..i) * n - tot * i`, which is `< 0`
// iff prefix `0..i` has average strictly below the global average `tot / n`,
// and `== 0` iff prefix `0..i` hits the global average exactly.
// The answer is `"Yes"` iff either:
// - `below`: some non-empty proper prefix `0..i` (`1 <= i < n`) dips strictly
//   below the global average (`diff(i) < 0`), or
// - `no_gap`: every element sits adjacent to an exact-average cut
//   (`diff(i) == 0 or diff(i + 1) == 0` for all `i in 0..n`).

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]0..=10 ** 9 = read()
let p = prefix(a)
let tot = a.sum()
fn diff(i: Int) -> Int: p.sum(..i) * n - tot * i

let below = any(diff(i) < 0 for i in 1..n)
let no_gap = all(diff(i) == 0 or diff(i + 1) == 0 for i in 0..n)
print(if below or no_gap: "Yes" else: "No")

// Source: Codeforces 2069C (Beautiful Sequence), https://codeforces.com/contest/2069/problem/C
// limits: 2000 ms, 250 MB
// Problem: n <= 2e5 values in 1..3. Count subsequences of length >= 3 where every element but
//   the first has a smaller one to its left and every element but the last a larger one to
//   its right, mod 998244353. Multitest, sum n <= 2e5.
// Solution: with values 1..3 these are exactly 1, then one or more 2s, then 3. Scan once:
//   ones counts 1s, open counts "1 then a nonempty set of 2s" (a 2 doubles open and adds
//   ones), and each 3 adds open to the answer. O(n).
// Bounds: n: 3..=200_000
@!multitest
type Z = Mod<998_244_353>

let n: 3..=200_000 = read()
var ones, open, ans: Z = 0
for a in read<[n]1..=3>():
    match a:
        1 => ones += 1
        2 => open = 2 * open + ones
        3 => ans += open
print(ans)

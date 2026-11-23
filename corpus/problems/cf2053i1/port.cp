// Source: Codeforces 2053I1 (Affectionate Arrays, easy version),
//   https://codeforces.com/contest/2053/problem/I1
// Problem: multitest; a_1..a_n (n <= 3e6, sum of n <= 3e6, |a_i| <= 1e9, max|a_i| <= sum
//   s). Among arrays b with a as a subsequence and sum s, take those of least boredom (max
//   subarray sum), then least length; print that length m.
// Solution: boredom s is the floor, and it holds exactly when every prefix sum of b lies in
//   [0, s]. Scan a keeping the interval of prefix sums still reachable; when it empties,
//   one inserted element before a_i resets it to all of [0, s]. b must end at s, which may
//   cost one more element. O(n).
@!multitest
let n: 1..=3e6 = read()
let a: [0..n]-1e9..=1e9 = read()
let s = a.sum()

var reach = 0..=0
var inserted = 0
for x in a:
    reach = max(reach.start + x, 0)..=min(reach.last + x, s)
    if reach.is_empty:
        inserted += 1
        reach = max(x, 0)..=min(s + x, s)
print(n + inserted + Int(s not in reach))

// Source: Codeforces 2201A2 (Lost Civilization, Hard Version),
//   https://codeforces.com/contest/2201/problem/A2
// limits: 2000 ms, 256 MB
// Problem: a generator repeatedly inserts x + 1 right after some element x; f(b) is the shortest
//   start sequence that can produce b. Given a of n <= 3e5 values (a_i <= 1e9), print the sum
//   of f over all subarrays. Multitest, sum n <= 3e5.
// Solution: start from the sum of all lengths, n(n + 1)(n + 2) / 6, and subtract each removal.
//   Take indices by decreasing a (ties left to right) on a linked list of survivors: i is
//   removed by its nearest surviving left neighbour p when a[p] = a[i] - 1, in every subarray
//   holding both, (p + 1)(n - i) of them; then unlink i. O(n log n) for the sort.
// Bounds: n: 1..=300_000; a: 1..=10 ** 9

@!multitest
let n: 1..=300_000 = read()
let a: [0..n]1..=10 ** 9 = read()

var prv: [0..n]0..=n = [0..n] i => if i > 0: i - 1 else: n  // n: none
var nxt: [0..n]0..=n = [0..n] i => i + 1                    // n: none
var ans = n * (n + 1) * (n + 2) / 6
for key in sorted([(10 ** 9 - a[i]) << 19 | i for i in 0..n]):  // a desc, then i (n < 2^19)
    let i = key % (1 << 19)
    let p = prv[i]
    if p < n and a[p] == a[i] - 1:
        ans -= (p + 1) * (n - i)
        nxt[p] = nxt[i]
        if nxt[i] < n: prv[nxt[i]] = p
print(ans)

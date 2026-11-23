// C. Problem from Ust-Katav
// https://codeforces.com/contest/2289/problem/C
// value p[i] is the minimum of [l, r] for a left choices of l and b of r; lengths with exactly one such subarray:
// all a + b - 1 lengths when min(a, b) == 1, else only the shortest and the longest
@!multitest
let n: 1..=2e5 = read()
let p: [0..n]1..=n = read()
let lt = p.prev_smaller()
let rt = p.next_smaller()
var ans = 0
for i in 0..n:
    let a = i - (lt[i] ?? -1)
    let b = (rt[i] ?? n) - i
    ans += if min(a, b) == 1: a + b - 1 else: 2
print(ans)

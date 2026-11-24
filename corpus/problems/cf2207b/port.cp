// Source: Codeforces 2207B (One Night At Freddy's), https://codeforces.com/contest/2207/problem/B
// limits: 1000 ms, 256 MB
// Problem: m animatronics start at danger 0; in each second 1..=l one of them (adversary's
//   choice) gains 1, and right after seconds a_1 < ... < a_n you reset one to 0. Print the
//   smallest x you can guarantee as the final maximum. n <= l, m * l <= 2e5; multitest, sum of
//   m * l <= 2e5.
// Solution: you always reset the largest; the adversary's best is to raise the smallest of the
//   animatronics that can still matter, at most (flashes left + 1) of them. Simulate with the
//   values kept in non-increasing order: increment the first minimum, and at a flash drop the
//   first value and add a 0 while the count allows. O(l * min(m, n + 1)).

@!multitest
let (n, m): (1..=2e5, 1..=2e5) = read()
let l: n..=2e5 / m = read()
let a: [0..n]1..=l = read()

var vals = [0 for _ in 0..min(m, n + 1)]
var idx = 0

for t in 1..=l:
    vals[vals.argmin()!] += 1
    if idx < n and t == a[idx]:
        idx += 1
        vals.remove(0)
        if vals.len < n - idx + 1: vals.push(0)

print(vals[0])

// Source: Codeforces 2066B (White Magic), https://codeforces.com/contest/2066/problem/B
// limits: 2000 ms, 250 MB
// Problem: a sequence is magical if min(a_1..a_i) >= mex(a_{i+1}..a_n) for every 1 <= i < n.
//   Given n <= 2e5 values 0 <= a_i <= 1e9, print the length of the longest magical subsequence.
//   Multitest, sum n <= 2e5.
// Solution: without zeros every suffix has mex 0, so all n values work. Two zeros never do
//   (a prefix min of 0 against a mex >= 1), so the answer is n - zeros or one more; the best one
//   to keep is the leftmost. Build that subsequence b and test it with prefix minima and suffix
//   mexes computed right to left over a seen array capped at |b|; O(n).
// Bounds: n: 1..=2e5; a: 0..=1e9
@!multitest
let n: 1..=2e5 = read()
let a = read<[n]0..=1e9>()
let first = a.find(0) ?? return print(n)
let b = [x for (i, x) in a.items if x != 0 or i == first]
let low = b.scan(min)
var seen: [0..=b.len]Bool = false
var mex: [0..=b.len]Int = 0
for i in (0..b.len).rev:
    if b[i] <= b.len: seen[b[i]] = true
    mex[i] = mex[i + 1]
    while seen[mex[i]]: mex[i] += 1
print(n - a.count(0) + Int((1..b.len).all(i => low.prefix(i) >= mex[i])))

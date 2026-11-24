// Source: Codeforces 2157A (Dungeon Equilibrium), https://codeforces.com/contest/2157/problem/A
// limits: 1000 ms, 256 MB
// Problem: n <= 100 values in 0..=n. An array is balanced when every value x in it occurs exactly
//   x times. Print the fewest deletions that make it balanced. Multitest.
// Solution: values are independent: with f copies of v, keep v of them when f >= v (delete f - v),
//   else delete all f (so every 0 goes). Count frequencies and sum the deletions, O(n).
// Bounds: n: 1..=100; a: 0..=n

@!multitest
let n: 1..=100 = read()
let a: [0..n]0..=n = read()
var freq: [0..=n]Int = 0
for x in a: freq[x] += 1
print(sum(if freq[v] < v: freq[v] else: freq[v] - v for v in 0..=n))

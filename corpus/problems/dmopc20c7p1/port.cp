// Source: DMOPC '20 Contest 7 P1 (Bob and Balance), https://dmoj.ca/problem/dmopc20c7p1
// Problem: 2n weights (n <= 5e5, masses <= 1e9) go on n two-plate scales, one weight per plate.
//   Maximise the tipped scales (unequal masses); print that number, then each scale's two
//   weight indices.
// Solution: sort the weights by mass and put the i-th on a scale with the (i + n)-th. A mass
//   held by f <= n weights cannot fill both ends, so only a mass with f > n balances, f - n
//   scales, which no pairing avoids. O(n log n).
// original: corpus/submissions/3686504.cpp
let n: 1..=5e5 = read()
domain Weight = 1..=2 * n
let mass: [Weight]1..=1e9 = read()
let order = Weight.sort(by: w => mass[w])
let pairs = [(order[i], order[i + n]) for i in 0..n]
print(count(mass[a] != mass[b] for (a, b) in pairs))
for (a, b) in pairs: print(a, b)

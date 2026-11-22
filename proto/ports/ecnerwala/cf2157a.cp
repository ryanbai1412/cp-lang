// Codeforces 2157 A — Dungeon Equilibrium
//
// An array is balanced iff every present value `v` occurs exactly `v` times.
// For each `v in 0..=n` with frequency `freq[v]`:
// - If `freq[v] < v`, we cannot add copies, so we must delete all `freq[v]`.
// - If `freq[v] >= v`, keeping `v` copies costs `freq[v] - v` deletions
//   (always at most deleting all `freq[v]`).

@!multitest
let n: 1..=100 = read()
let a: [0..n]0..=n = read()
var freq: [0..=n]Int = 0
for x in a: freq[x] += 1
print(sum(if freq[v] < v: freq[v] else: freq[v] - v for v in 0..=n))

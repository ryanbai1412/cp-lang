// Codeforces 2152 A — Increase or Smash
//
// Each distinct value in `a` requires one rise operation and (except the last)
// one smash/drop operation, giving `2 * distinct - 1` operations total.

@!multitest
let n: 1..=100 = read()
let a: [0..n]1..=100 = read()
print(2 * Set(a).len - 1)

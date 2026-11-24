// Tuned: a Fenwick tree over the compressed scores counts the earlier games at or
// below each score, in place of an ordered multiset.
//
// Source: CCC 2005 S5 (Pinball Ranking), https://dmoj.ca/problem/ccc05s5
// Problem: t <= 1e5 game scores in 0..1e9, in order. After each game its rank is one more than
//   the number of games so far with a strictly higher score. Print the average rank (abs error
//   1e-2; printed to two decimals). Scores may repeat.
// Solution: keep the earlier scores in an ordered multiset; the number above s is
//   len - (count below s + 1). Sum 1 + that and print the exact average with `fixed`
//   (no floats). O(t log t).
// Bounds: t: 1..=1e5; scores: 0..=1e9
let t: 1..=1e5 = read()
let scores: [t]0..=1e9 = read()

let (domain Level, level) = compress(scores)
var seen = Fenwick<Level, Int>()
var ranks = 0
for (i, s) in scores.items:
    // the i earlier games less those at or below s
    ranks += 1 + i - seen.prefix(level.index_of(s).r)
    seen.add(level.index_of(s), 1)
print(fixed(ranks, t, 2))

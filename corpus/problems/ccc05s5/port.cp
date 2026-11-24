// Source: CCC 2005 S5 (Pinball Ranking), https://dmoj.ca/problem/ccc05s5
// Problem: t <= 1e5 game scores in 0..1e9, in order. After each game its rank is one more than
//   the number of games so far with a strictly higher score. Print the average rank (abs error
//   1e-2; printed to two decimals). Scores may repeat.
// Solution: keep the earlier scores in an ordered multiset; the number above s is
//   len - (count below s + 1). Sum 1 + that and print the exact average with `fixed`
//   (no floats). O(t log t).
let t: 1..=1e5 = read()
let scores: [t]0..=1e9 = read()

var seen = SortedBag<Int>()
var ranks = 0
for s in scores:
    ranks += 1 + seen.len - seen.rank(s + 1)
    seen.add(s)
print(fixed(ranks, t, 2))

// Source: AtCoder Educational DP Contest L (Deque), https://atcoder.jp/contests/dp/tasks/dp_l
// limits: 2000 ms, 1024 MB
// Problem: n <= 3000 numbers in a row. Two players alternately take the leftmost or
//   rightmost one; each maximises (own total - other's). Print first minus second.
// Solution: lead[r], the best margin for the player to move when the range r is left, is
//   max(a[first] - lead[r without first], a[last] - lead[r without last]). `I.ranges` goes by
//   length, so both smaller ranges are done first: O(n^2) states, O(1) each.
// original: corpus/submissions/4131776.cpp
let n: 1..=3000 = read()
domain I = 0..n
let a: [I]1..=1e9 = read()

// lead[r]: the best margin for the player to move when the range r is left
var lead: [I.ranges]Int = 0
for r in I.ranges:
    lead[r] = if r.len == 1: a[r.first] else: max(a[r.first] - lead[r.drop(1)], a[r.last] - lead[r.drop_last(1)])
print(lead[I.range])

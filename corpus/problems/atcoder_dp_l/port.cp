// Source: AtCoder Educational DP Contest L (Deque), https://atcoder.jp/contests/dp/tasks/dp_l
// Problem: n <= 3000 numbers in a row. Two players alternately take the leftmost or
//   rightmost one; each maximises (own total - other's). Print first minus second.
// Solution: lead(l, r), the best margin for the player to move on a[l..=r], is
//   max(a[l] - lead(l + 1, r), a[r] - lead(l, r - 1)): O(n^2) states, O(1) each.
// original: corpus/submissions/4131776.cpp
let n: 1..=3000 = read()
domain I = 0..n
let a: [I]1..=1e9 = read()

// lead(l, r): the best margin for the player to move when a[l..=r] is left.
@memo
fn lead(l: I, r: I) -> Int:
    if l >= r: return a[l]
    max(a[l] - lead(l + 1, r), a[r] - lead(l, r - 1))
print(lead(I.first!, I.last!))

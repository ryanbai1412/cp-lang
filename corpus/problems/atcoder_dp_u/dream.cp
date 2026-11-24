// Source: AtCoder Educational DP Contest U (Grouping), https://atcoder.jp/contests/dp/tasks/dp_u
// Problem: n <= 16 rabbits with symmetric scores |a[i][j]| <= 1e9, a[i][i] = 0. Split them into
//   groups, earning a[i][j] for every pair in one group; print the largest total.
// Solution: score[g], what subset g earns as one group, sums its pairs: O(2^n n^2). Then
//   best[s] = max over nonempty subsets g of s of score[g] + best[s - g], g being the group
//   that stays together; walking the subsets of every subset costs O(3^n).
// original: corpus/submissions/4131872.cpp
let n: 1..=16 = read()
domain R = 0..n
type Group = Subset<R>
let a: [R, R]-1e9..=1e9 = read()

// score[g]: what the rabbits of g earn as one group
var score: [Group]Int = 0
for g in Group:
    score[g] = sum(a[i, j] for i in g for j in g if i < j)

// best[s]: the best split of s. Every non-empty g inside s is tried as the
// group of s's rabbits that stay together.
var best: [Group]Int = 0
for s in Group:
    for g in s.subsets if g != Group.empty: best[s] max= score[g] + best[s - g]
print(best[Group.full])

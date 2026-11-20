// Educational DP U, Grouping: split n rabbits into groups. A group scores
// a[i][j] for each pair inside it. Maximise the total.
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
    var g = s.idx
    while g > 0:
        best[s] max= score[Group(g)] + best[Group(s.idx - g)]
        g = (g - 1) & s.idx
print(best[Group.full])

// AtCoder DP U Grouping: split n rabbits into groups; every pair inside a
// group scores like[a, b], possibly negative. Maximise the total score.
let n: 1..=16 = read()
domain Rabbit = 0..n
type Group = Subset<Rabbit>
let like: [Rabbit, Rabbit]-1e9..=1e9 = read()

let score: [Group]Int = [Group] g => sum(like[a, b] for a in g for b in g if a < b)
// best[s]: the best split of the rabbits in s; one group is g, the rest is split best
var best = score
for s in Group:
    for g in s.subsets: best[s] max= score[g] + best[s - g]
print(best[Group.full])

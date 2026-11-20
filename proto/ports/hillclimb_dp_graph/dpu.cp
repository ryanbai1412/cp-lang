// AtCoder DP U, Grouping: split n rabbits into groups; every pair inside a
// group scores like[i, j] (possibly negative). Maximise the total score.
let n: 1..=16 = read()
domain Rabbit = 0..n
type Group = Subset<Rabbit>
let like: [Rabbit, Rabbit]-1e9..=1e9 = read()

fn together(g: Group) -> Int:
    sum(like[a, b] for a in g for b in g if a < b)

let score: [Group]Int = [Group] g => together(g)

// best[s]: the best split of the rabbits in s
var best: [Group]Int = 0
for s in Group:
    best[s] = score[s]
    for g in s.subsets:
        best[s] max= score[g] + best[s - g]
print(best[Group.full])

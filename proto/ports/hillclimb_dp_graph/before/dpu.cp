// cp: 0.1 0f8d77b
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
    var g = s.idx                         // walk the submasks of s by hand
    while g > 0:
        best[s] max= score[Group.nth(g)] + best[Group.nth(s.idx ^ g)]
        g = (g - 1) & s.idx
print(best[Group.full])

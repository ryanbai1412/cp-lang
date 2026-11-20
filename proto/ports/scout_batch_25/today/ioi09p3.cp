// IOI '09 P3, POI (buildable today-form)
// original: corpus/submissions/5106576.cpp
let N: 1..=2000 = read()
let T: 1..=2000 = read()
let P: 1..=N = read()
domain C = 0..N
domain Task = 0..T
let solved: [C, Task]0..=1 = read()

var worth: [Task]Int = 0
for c in C:
    for t in Task:
        worth[t] += 1 - solved[c, t]

// (-score, -tasks solved, id): ascending order is the ranking
var ranking: List<(Int, Int, Int)> = []
for c in C:
    var score = 0
    var tasks = 0
    for t in Task:
        if solved[c, t] == 1:
            score += worth[t]
            tasks += 1
    ranking.push((-score, -tasks, c.to_idx + 1))
sort(mut ranking)

for r in 0..N:
    if ranking[r].2 == P:
        print(-ranking[r].0, r + 1)

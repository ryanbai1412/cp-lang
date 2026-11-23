// Source: IOI 2009 P3 (POI), https://dmoj.ca/problem/ioi09p3
// Problem: N <= 2000 contestants, T <= 2000 tasks, a 0/1 table of who solved what. A task is worth
//   the number who did not solve it. Rank by score, then tasks solved (both descending), then ID.
//   Print the score and the rank of contestant P.
// Solution: count each task's worth in one pass over the table, then each contestant's score and
//   solved count; sort the keys (-score, -solved, id) and find P. O(N T + N log N).
// original: corpus/submissions/5106576.cpp
let n: 1..=2000 = read()
let tasks: 1..=2000 = read()
let p: 1..=n = read()
domain C = 0..n
domain Task = 0..tasks
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
ranking.sort()

for r in 0..n:
    if ranking[r].2 == p:
        print(-ranking[r].0, r + 1)

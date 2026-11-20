// IOI '09 P3 - POI (DMOJ: ioi09p3)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// N contestants, T tasks. A task is worth the number of contestants who did
// NOT solve it. Rank by score (high first), then by number of tasks solved
// (high first), then by id (low first). Print the score and the rank of
// contestant P.
let (N, T, P): (1..=2000, 1..=2000, 1..=N) = read()
domain C = 1..=N
domain Task = 0..T
let solved: [C, Task]Bool = read()

let worth = [Task] t => count(c for c in C if not solved[c, t])
let score = [C] c => sum(worth[t] for t in Task if solved[c, t])
let tasks = [C] c => count(t for t in Task if solved[c, t])

let ranking = sorted(C, by: c => (desc(score[c]), desc(tasks[c]), c))
print(score[C(P)], ranking.position(C(P)) + 1)

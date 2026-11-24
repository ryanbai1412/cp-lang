// Source: IOI 2009 P3 (POI), https://dmoj.ca/problem/ioi09p3
// limits: 1000 ms, 64 MB
// Problem: N <= 2000 contestants, T <= 2000 tasks, a 0/1 table of who solved what. A task is worth
//   the number who did not solve it. Rank by score, then tasks solved (both descending), then ID.
//   Print the score and the rank of contestant P.
// Solution: each task is worth the number who missed it, then each contestant has a score and
//   a solved count; P's rank is one plus the contestants whose key (-score, -solved, id) is
//   smaller. O(N T).
// original: corpus/submissions/5106576.cpp
let (n, tasks, p): (1..=2000, 1..=2000, 1..=2000) = read()
domain C = 1..=n
domain Task = 1..=tasks
let philip = C(p)
let solved: [C, Task]0..=1 = read()

let worth = [Task] t => n - sum(solved[c, t] for c in C)
let score = [C] c => sum(solved[c, t] * worth[t] for t in Task)
let done = [C] c => sum(solved[c, t] for t in Task)
// ranking order: higher score, then more tasks, then lower id
let key = (c: C) => (-score[c], -done[c], c)
print(score[philip], 1 + count(key(c) < key(philip) for c in C))

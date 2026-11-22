// ioi09p3 POI: a task is worth the number of contestants who missed it; rank by score, then by
// tasks solved, then by id. Print contestant p's score and rank.
let (n, t) = read<(1..=2_000, 1..=2_000)>()
let p = read<1..=n>()
domain C = 1..=n
domain T = 1..=t
let solved = read<[C, T]0..=1>()
let worth = [T] j => n - C.map(c => solved[c, j]).sum()
let score = [C] c => T.map(j => solved[c, j] * worth[j]).sum()
let key = c => (-score[c], -solved[c].sum(), c)
print(score[p], C.count(c => key(c) < key(p)) + 1)

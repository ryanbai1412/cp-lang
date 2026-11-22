// IOI 2009 Poi: each task is worth the number of contestants who missed it; rank by score, then
// tasks solved, then id. Print Philip's score and rank.
let (n, t, philip) = read (Int, Int, Int)
domain C = 1..=n
domain T = 1..=t
let solved = read [C, T]Bool
let worth = [for task in T: C.count(c => not solved[c, task])]
let score = [for c in C: T.sum(of: task => if solved[c, task]: worth[task] else: 0)]
let rank_key = (c: C) => (desc(score[c]), desc(T.count(task => solved[c, task])), c)
print(score[philip], C.count(c => rank_key(c) <= rank_key(philip)))

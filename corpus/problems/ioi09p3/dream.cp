// IOI 2009 Poi: each task is worth the number of contestants who missed it; rank by score, then
// tasks solved, then id. Print Philip's score and rank.
let (n, t, p): (1..=2000, 1..=2000, 1..=2000) = read()
domain C = 1..=n
domain T = 1..=t
let philip = C(p)
let row = read<[C, T]0..=1>()
let solved = [C, T] (c, task) => row[c, task] == 1
let worth = [T] task => C.count(c => not solved[c, task])
let score = [C] c => T.filter(task => solved[c, task]).sum(of: task => worth[task])
let rank_key = (c: C) => (desc(score[c]), desc(T.count(task => solved[c, task])), c)
print(score[philip], C.count(c => rank_key(c) <= rank_key(philip)))

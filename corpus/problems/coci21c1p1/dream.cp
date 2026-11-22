# COCI 2021/22 C1 P1: a basket is worth 150 when the same player scored within the last 10 seconds,
# else 100. Players 1..=4 are team A, 5..=8 team B. Print both totals.
domain Player = 1..=8
var last: [Player]?Int = none
var score = [0, 0]
for (t, p, _) in read [n] (Int, Player, Int):
    let worth = if last[p].is_some_and(s => t - s <= 10): 150 else: 100
    last[p] = some(t)
    score[if p <= 4: 0 else: 1] += worth
print(score)

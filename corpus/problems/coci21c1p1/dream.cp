// coci21c1p1 Ljestvica: each basket is worth 100, or 150 when the same player scored within the
// last 10 seconds. Players 1..=4 are team A, 5..=8 team B.
let n = read<1..=1_000>()
var last: [1..=8]?Int = none
var score: [2]Int = 0
for (t, x, _) in read<[n](Int, 1..=8, Int)>():
    var points = 100
    if last[x].map(l => t - l <= 10) ?? false:
        points = 150
    last[x] = t
    score[if x <= 4: 0 else: 1] += points
print(score)

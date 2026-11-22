// coci21c1p1 Ljestvica: each basket is worth 100, or 150 when the same player scored within the
// last 10 seconds. Players 1..=4 are team A, 5..=8 team B.
let n = read<1..=1_000>()
enum Team: Home, Away
domain Player = 1..=8
var last: [Player]?Int = none
var score: [Team]Int = 0
for (t, x, _) in read<[n](Int, Player, Int)>():
    var points = 100
    if last[x].map(l => t - l <= 10) ?? false:
        points = 150
    last[x] = t
    score[if x <= 4: Team.Home else: Team.Away] += points
print(score)

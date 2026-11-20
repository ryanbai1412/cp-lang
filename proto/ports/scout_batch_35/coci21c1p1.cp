// COCI 2021/22 #1 Ljeto: n hits in a water fight, in time order, each
// "at second t player a sprayed player b". Players 1..=4 are one team and
// 5..=8 the other. A hit is worth 100 points, and 150 when the same player
// also scored within the last 10 seconds. Print both teams' points.
// original: corpus/submissions/4080824.cpp
let n: 1..=100 = read()
domain Player = 1..=8
var last: [Player]?Int = none
var score: [Bool]Int = 0                        // indexed by "is on the second team"
for _ in 0..n:
    let (t, who, _): (Int, Player, Player) = read()
    let streak = if let before = last[who]: t - before <= 10 else: false
    score[who.to_idx >= 4] += if streak: 150 else: 100
    last[who] = t
print(score[false], score[true])

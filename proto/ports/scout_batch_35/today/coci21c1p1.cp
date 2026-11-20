// Ljeto, in the forms proto/cpc builds today. See ../coci21c1p1.cp.
// original: corpus/submissions/4080824.cpp
let n: Int = read()
domain Player = 1..=8
var last: [Player]?Int = none
var first = 0
var second = 0
for _ in 0..n:
    let t: Int = read()
    let who: Player = read()
    let hit: Player = read()
    var points = 100
    if let before = last[who]:
        if t - before <= 10: points = 150
    if who.ord < 4: first += points
    else: second += points
    last[who] = t
print(first, second)

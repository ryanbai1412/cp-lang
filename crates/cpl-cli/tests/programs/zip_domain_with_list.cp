enum Side: L | R
domain Round = 0..3
let sides: List<Side> = [Side.L, Side.R, Side.L]
var rounds: [Round, Side]Int = 0
for (r, s) in zip(Round, sides):
    rounds[r, s] += 1
print(rounds[Round.first!, Side.L])
for (r, s) in zip(sides, Round):
    rounds[s, r] += 2
print(rounds[Round(1), Side.R], rounds[Round(2), Side.L])

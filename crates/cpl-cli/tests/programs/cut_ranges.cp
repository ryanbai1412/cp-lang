domain V = 0..5
fn cost(piece: Range<V>) -> Int = piece.len * piece.len
var c: V.cuts = 0
c = 3
var dp: [V.cuts]Int = 1000
dp[0] = 0
for i in V.cuts:
    for j in V.cuts.first!..i:
        dp[i] min= dp[j] + cost(j..i)
let a: [V]Int = 1
let s = a[V.cuts.first!..V.cuts.last!]
print(c.ord, dp[V.cuts.last!], s.len)

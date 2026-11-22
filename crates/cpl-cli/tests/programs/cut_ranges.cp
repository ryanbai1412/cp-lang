domain V = 0..5
fn cost(piece: Range<V>) -> Int = piece.len * piece.len
var c: V.bounds = 0
c = 3
var dp: [V.bounds]Int = 1000
dp[0] = 0
for i in V.bounds:
    for j in V.bounds.first..i:
        dp[i] min= dp[j] + cost(j..i)
let a: [V]Int = 1
let s = a[V.bounds.first..V.bounds.last]
print(c, dp[V.bounds.last], s.len)

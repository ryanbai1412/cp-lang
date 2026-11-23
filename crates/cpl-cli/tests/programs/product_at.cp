// `P.at((x1, ..., xn))` on a product takes its parts tuple and is `None`
// when any part is outside its factor, never a neighbouring row (R04.2.10).
domain R = 0..3
domain C = 0..3
domain P = (R, C)
print(P((1, 2)))
print(P.at((1, 2))!)
print(yes(P.at((3, 0)) == None), yes(P.at((0, -1)) == None))
var n = 0
for (r, c) in P:
    if let q = P.at((Int(r), Int(c) + 1)): n += q.ord
print(n)

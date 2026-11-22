struct Id(v: Int)
struct Edge(to: Int, w: Int)

let n: Int = read()
domain V = 0..n
struct Hop(at: V, cost: Int)

var best: ?Edge = none
print(yes(best == none))
for i in 0..3:
    let e = Edge(i, 10 - i)
    if best == none or (best ?? e).w > e.w:
        best = e
if let e = best:
    print(e.to, e.w)
let ids: List<?Id> = [Id(1), none, Id(3)]
print(yes(ids[0] == ids[2]), yes(ids[1] == none))
var far: ?Hop = none
far = Hop(V.first, 7)
if let h = far:
    print(Int(h.at), h.cost)
print((ids[1] ?? Id(9)).v)

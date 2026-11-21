struct Edge: to: Int, w: Int
struct Pt: x, y: Int
struct Seg: l, r: Int, sum: Int = 0

let e = Edge(to: 3, w: 7)
let p = Pt(x: 4, y: 5)
let s = Seg(l: 1, r: 9)
print(e.to + e.w)
print(p.x * p.y)
print(s.l, s.r, s.sum)

// expect: E0303 at 2
// rule: 02.6.2
struct Edge: u: Int, v: Int, w: Int
let e = Edge(1, 2)
print(e.u)

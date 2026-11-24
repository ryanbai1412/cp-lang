// expect: E0303
// rule: 02.6.2
struct Edge: u: Int, v: Int, w: Int
let e = Edge(1, 2) // <- here
print(e.u)

use std.grid.{adj4}
domain R = 0..3
domain C = 0..3
let wall: [R, C]Bool = [R, C] (r, c) => r.ord == 1 and c.ord < 2
let d = bfs(p => [q for q in p.adj4 if not wall[q]], [(R(0), C(0))])
print(d[(R(2), C(0))], d[(R(2), C(2))])
let e = dijkstra(p => [q for q in p.adj4 if not wall[q]], [((R(0), C(0)), 0)], q => 2)
print(e[(R(2), C(0))])

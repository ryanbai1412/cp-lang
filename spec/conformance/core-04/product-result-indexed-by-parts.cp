// expect: ok, output "3\nsame\napart"
// rule: 04.2.11
// A result keyed by an inline product `(R, C)` takes `comp[r, c]` as well as `comp[(r, c)]` (decision 159; round 27, F2/F3).
use std.graph.{components}
domain R = 0..2
domain C = 0..3
let wall: [R, C]Bool = [R, C] (r, c) => Int(c) == 1
let (domain K, comp) = components((p: (R, C)) => p.adj4.filter(q => wall[q] == wall[p]))
print(K.count)
if comp[R(0), C(0)] == comp[(R(1), C(0))]: print("same")
if comp[R(0), C(0)] != comp[R(1), C(2)]: print("apart")

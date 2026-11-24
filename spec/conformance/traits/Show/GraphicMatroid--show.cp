// expect: ok, output "0-2 2-3 3-0"
use std.flow.{GraphicMatroid}
domain V = 0..4
domain E = 0..3
let es: [E](u: V, v: V) = [(u: V(0), v: V(2)), (u: V(2), v: V(3)), (u: V(3), v: V(0))]
print(GraphicMatroid(es))

use std.graph.{components}
domain V = 0..4
let g: [V]List<V> = [[2], [], [0], []]
let (domain C, comp) = components(g)
print(C.count)

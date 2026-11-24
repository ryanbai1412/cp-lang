use std.geometry.{Vec}

// A key with an F64 inside has no Hash, so its table stays a tree.
let p1 = Vec(1.0, 2.0)
let p2 = Vec(3.0, 4.0)
var id: Map<Vec, Int> = [p1: 1, p2: 2]
id[Vec(0.5, 0.0)] = 3
var seen: Set<(F64, Int)> = Set()
seen.add((0.5, 1))
seen.add((0.5, 1))
print(id[p1], id[p2], id[Vec(0.5, 0.0)], id.len, seen.len)

// Each of m roads can be kept or reversed; road i must not point the same
// way as road j for every listed conflict. Print one valid choice or "NO".
// "These two differ" is an edge of a graph that must stay two-colourable.
use std.dsu.{ParityDsu}

let (m, k): (Int, Int) = read()
domain Road = 1..=m
var d = ParityDsu<Road>()
for _ in 0..k:
    let (a, b): (Int, Int) = read()
    d.unite(Road(a), Road(b))
if d.consistent:
    print("YES")
    print([Int(d.find(r).flip) for r in Road])
else:
    print("NO")

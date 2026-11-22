// expect: E0602 at 14
// rule: 05.9.4
domain V = 1..=4
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
use std.graph.{Graph, dfs}
domain E = 0..3
let es: [E](u: V, v: V) = [(u: 1, v: 2), (u: 2, v: 3), (u: 3, v: 4)]
let g = Graph.undirected(es)
let s = V.first!
var seen: [V]Bool = false
for step in dfs(g, from: [s], seen: mut seen):
    seen[s] = true          // error: seen is lent `mut` to the dfs for the whole loop

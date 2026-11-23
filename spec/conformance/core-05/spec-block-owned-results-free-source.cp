// expect: ok, output "2"
// rule: 05.3.5
domain V = 1..=4
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
domain E = 0..3
let es: [E](u: V, v: V) = [(u: 1, v: 2), (u: 2, v: 3), (u: 3, v: 4)]
let g = Graph.undirected(es)
let s = V.first!
let xs = List(1, 2, 3)
let p = (x: Int) => x > 1
let t = Rooted(g, root: r)        // reads g, owns its arrays: g is free
let fast = LcaTable(t)            // reads t, owns its table: t is free
let ev = xs.filter(p)             // filter is `lends`: xs is lent while ev lives
print(ev.count())

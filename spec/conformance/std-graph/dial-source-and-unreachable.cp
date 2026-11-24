// expect: ok, output "YES YES"
domain V = 0..3
domain E = 0..1
let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 2)]
let p = dial(Graph.directed(es), from: [V(0)], max_w: 2)
print(yes(p[0] == Some(0)), yes(p[2] == None))

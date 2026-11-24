// expect: ok, output "1"
domain V = 0..2
domain E = 0..1
let es: [E](u: V, v: V) = [(u: V(0), v: V(1))]
let (domain B, block) = blocks(Graph.undirected(es))
print(B.count)

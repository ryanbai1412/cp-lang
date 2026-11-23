// expect: ok, output "0 0 1\n1 2 2\n0 1 0\n1 2 2"
domain V = 0..3
domain E = 0..2
let es: [E](u: V, v: V) = [(u: 1, v: 0), (u: 2, v: 2)]
print([(Int(x.e), Int(x.u), Int(x.v)) for x in Graph.undirected(es).edges()])
print([(Int(x.e), Int(x.u), Int(x.v)) for x in Graph.directed(es).edges()])

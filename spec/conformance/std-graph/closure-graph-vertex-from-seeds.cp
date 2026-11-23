// expect: ok, output "3"
// rule: std.graph.bfs
// With a closure as the graph, the vertex type comes from `from:`; the lambda needs no annotation.
domain V = 0..5
let p = V(0)
let d = bfs(v => [w for w in V if Int(w) == Int(v) + 1], from: [p])
print(d[V(3)] ?? -1)

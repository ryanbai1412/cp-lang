// expect: ok, output "0 0"
// rule: std.flow.FlowGraph.new
// `FlowGraph<V>()` starts every vertex with an empty arc list.
let g = FlowGraph<0..3>()
print(g.arcs.count(), g.out(1).count())

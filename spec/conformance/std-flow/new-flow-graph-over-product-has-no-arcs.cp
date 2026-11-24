// expect: ok, output "0 0\n5"
// rule: std.flow.FlowGraph.new
// `FlowGraph<(A, B)>()` over a product of domains starts every vertex with an empty arc list; its
// table of arc lists takes the cell type from the struct field, so std writes `[V] v => []`.
domain A = 0..2
domain B = 0..3
var g = FlowGraph<(A, B)>()
print(g.arcs.count(), g.out((A(1), B(2))).count())
g.add((A(0), B(0)), (A(1), B(2)), Some(5))
print(g.max_flow((A(0), B(0)), (A(1), B(2))))

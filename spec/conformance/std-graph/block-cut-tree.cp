// expect: ok, output "2 2 0 1 1 1"
// rule: std.graph.block_cut_tree

use std.graph.{block_cut_tree}

domain V = 0..3
let g: [V]List<V> = [[V(1)], [V(0), V(2)], [V(1)]]
let (domain B, links) = block_cut_tree(g)
print(B.count, links.len, links[0].b, links[0].v, links[1].b, links[1].v)

// expect: ok, output "YES 9"
// rule: std.strings.suffix_tree_edges
use std.strings.{SuffixTreeEdge, suffix_tree_edges}
let e1 = suffix_tree_edges("aa")
let ok1 = e1 == [SuffixTreeEdge(0, 1, 1, 1, 1..2), SuffixTreeEdge(1, 2, 0, 1, 0..2)]
let e2 = suffix_tree_edges("ababa")
let dist = [e.edge_len for e in e2].sum()
print(yes(ok1), dist)

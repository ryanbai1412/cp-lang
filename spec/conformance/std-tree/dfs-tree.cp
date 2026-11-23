// expect: ok, output "YES\nYES\nYES"
// `g.dfs_tree(root)` is the DFS tree and its edge mask: a non-tree edge of an undirected graph
// joins a vertex to an ancestor, and a vertex the search never reaches is a root of its own.
use std.tree.*
domain V = 0..5
domain E = 0..4
let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2), (u: 2, v: 0), (u: 0, v: 3)]
let (tree, tree_edge) = Graph.undirected(es).dfs_tree(0)
print(yes(tree.parent(2) == Some(1) and tree.depth(2) == 2 and tree.parent(3) == Some(0)))
print(yes(tree_edge == [true, true, false, true]))
print(yes(tree.parent(4) == None and tree.depth(4) == 0))

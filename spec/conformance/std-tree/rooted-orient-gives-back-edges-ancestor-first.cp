// expect: ok, output "2 0\n3 1\n0 0"
// rule: std.tree.Rooted.orient
// Every edge `dfs_tree` leaves out of an undirected graph's tree joins a vertex to an ancestor,
// so `orient` gives it ancestor first, whichever way the edge was written; a tree edge too.
domain V = 0..4
domain E = 0..5
let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2), (u: 0, v: 2), (u: 2, v: 3), (u: 3, v: 1)]
let (tree, tree_edge) = Graph.undirected(es).dfs_tree(V(0))
for e in E:
    if not tree_edge[e]:
        let o = tree.orient(es[e].u, es[e].v)
        print(o.desc.ord, o.anc.ord)
let o = tree.orient(V(0), V(0))
print(o.anc.ord, o.desc.ord)

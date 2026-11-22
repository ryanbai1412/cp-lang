// EGOI 2023 P7 Find the Tree: build a rooted spanning tree that uses every required arc and no
// banned arc, or print NO. The required arcs must form a forest; each of its trees is a block
// that hangs under another block. Block c may hang under block k unless every vertex of k is a
// banned parent of c's root. The blocks then need a spanning tree in the complement of the
// "may not" graph, found by DFS over unvisited blocks.
domain V = [read Int]
let (m1, m2) = read (Int, Int)
let need = read [m1](child: V, parent: V)
let banned = read [m2](child: V, parent: V)

let Some(forest) = Rooted.from_parents(need) else: print("NO"); return
domain B = forest.roots
let block = forest.root_of
let banned_parents = V.group(banned, by: .child).map(es => es.map(.parent).collect::<Set>())
let cannot = Graph<B>([for c in B
                       for (k, hits) in banned_parents[c].tally(by: v => block[v])
                       if hits == forest.subtree(k).len: (u: k, v: c)])
let hang = cannot.complement().dfs_forest()
if hang.roots.len != 1: print("NO"); return

for e in need: print(e.parent, e.child)
for (k, c) in hang.tree_arcs:
    print(forest.subtree(k).find(v => v not in banned_parents[c])!, c)

// SDOI 2017 "Tree Painting": every node starts with its own colour. Paint a
// root path with a fresh colour; count colours on a path; maximize the colour
// count to the root over a subtree. Painting is a link-cut "access".
let (n, q) = read<(1..=1e5, 1..=1e5)>()
domain V = 1..=n
enum Op = @tag(1) Paint(x: V) | @tag(2) Path(x: V, y: V) | @tag(3) Deepest(x: V)
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), root: V.first!)
let lca = LcaTable(t)
var colours = SegTree(max, t.reorder([V] v => t.depth(v) + 1), act: add)
var paths = PreferredPaths(t)

for op in read<[q]Op>():
    match op:
        Paint(x) =>
            for (lost, gained) in paths.access(x):
                if let top = lost: colours.apply(t.sub(top), 1)
                if let g = gained: colours.apply(t.sub(g), -1)
        Path(x, y) =>
            let at = (v: V) => colours.get(t.pos(v))
            print(at(x) + at(y) - 2 * at(lca.lca(x, y)) + 1)
        Deepest(x) =>
            print(colours.fold(t.sub(x)))

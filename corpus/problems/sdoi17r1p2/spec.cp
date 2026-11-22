// SDOI 2017 "Tree Painting": every node starts with its own colour. Paint a
// root path with a fresh colour; count colours on a path; maximize the colour
// count to the root over a subtree. Colour classes are preferred paths, so
// painting is an access, and each path switch shifts one subtree's counts.
enum Op = Paint(x: V) | Path(x: V, y: V) | Deepest(x: V)

let (n, q) = read<(1..=1e5, 1..=1e5)>()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), root: V.first!)
let lca = LcaTable(t)
var colours = SegTree(max, [t.Pre] p => t.depth(t.at(p)) + 1, act: add)
var paths = PreferredPaths(t)

for op in read<[q]Op>():
    match op:
        Paint(x) =>
            for (lost, gained) in paths.access(x):
                if lost is Some(top):
                    colours.apply(t.sub(top), 1)
                colours.apply(t.sub(gained), -1)
        Path(x, y) =>
            let at = (v: V) => colours.fold(t.pos(v)..=t.pos(v))
            print(at(x) + at(y) - 2 * at(lca.lca(x, y)) + 1)
        Deepest(x) =>
            print(colours.fold(t.sub(x)))

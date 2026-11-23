// SDOI 2017 "Tree Painting": every node starts with its own colour. Paint a
// root path with a fresh colour; count colours on a path; maximize the colour
// count to the root over a subtree. Colour classes are preferred paths, so
// painting is an access, and each path switch shifts one subtree's counts.
let (n, q) = read<(1..=1e5, 1..=1e5)>()
domain V = 1..=n
// payloads are `1..=n`, not `V`: a domain-typed payload misreads the tag (compiler worklog)
enum Op = @tag(1) Paint(x: 1..=n) | @tag(2) Path(x: 1..=n, y: 1..=n) | @tag(3) Deepest(x: 1..=n)
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), root: V.first!)
let lca = LcaTable(t)
var colours = SegTree(max, t.reorder([V] v => t.depth(v) + 1), act: add)
var paths = PreferredPaths(t)
// `access` doesn't yield the cut below `x` (stdlib worklog), so track each preferred child
var below: [V]?V = None

for op in read<[q]Op>():
    match op:
        Paint(x) =>
            if let c = below[V(x)]: colours.apply(t.sub(c), 1)
            below[V(x)] = None
            for (lost, gained) in paths.access(V(x)):
                if let top = lost: colours.apply(t.sub(top), 1)
                colours.apply(t.sub(gained), -1)
                below[t.parent(gained)!] = Some(gained)
        Path(x, y) =>
            let at = (v: V) => colours.fold(t.pos(v)..=t.pos(v))
            print(at(V(x)) + at(V(y)) - 2 * at(lca.lca(V(x), V(y))) + 1)
        Deepest(x) =>
            print(colours.fold(t.sub(V(x))))

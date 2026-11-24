// Source: SDOI 2017 Round 1 P2 (Tree Colouring), https://dmoj.ca/problem/sdoi17r1p2
// Problem: a tree on n <= 1e5 vertices rooted at 1, every vertex its own colour; m <= 1e5 ops:
//   "1 x" paints the path root..x a brand-new colour, "2 x y" prints the number of colours on
//   path x..y, "3 x" prints the most colours on a path from the root to a vertex in sub(x).
// Solution: only root paths are painted, so each colour is a preferred path and painting x is
//   an access of x in PreferredPaths. f[u] = colours on root..u lives in a range add, range
//   max SegTree over preorder; each switch of a preferred child adds 1 to the subtree it
//   leaves and takes 1 from the one it joins (amortised O(log n) switches, each O(log n)).
//   Query 2 is f[x] + f[y] - 2 f[lca] + 1, query 3 the max of f over sub(x).
// original: corpus/submissions/5463990.cpp
let (n, q) = read<(1..=1e5, 1..=1e5)>()
domain V = 1..=n
enum Op = @tag(1) Paint(x: 1..=n) | @tag(2) Path(x: 1..=n, y: 1..=n) | @tag(3) Deepest(x: 1..=n)
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), root: V.first!)
let lca = LcaTable(t)
var colours = SegTree(max, t.reorder([V] v => t.depth(v) + 1), act: add)
var paths = PreferredPaths(t)

for op in read<[q]Op>():
    match op:
        Paint(x) =>
            for (lost, gained) in paths.access(V(x)):
                if let top = lost: colours.apply(t.sub(top), 1)
                if let g = gained: colours.apply(t.sub(g), -1)
        Path(x, y) =>
            let at = (v: V) => colours.get(t.pos(v))
            print(at(V(x)) + at(V(y)) - 2 * at(lca.lca(V(x), V(y))) + 1)
        Deepest(x) =>
            print(colours.fold(t.sub(V(x))))

// SDOI '17 R1 P2 - Tree Colouring: a tree rooted at 1 where every vertex
// starts with its own colour. "1 x" paints the path from the root to x in a
// brand new colour; "2 x y" prints the number of colours on the path x..y;
// "3 x" prints, over every u in the subtree of x, the largest number of
// colours on the path from the root to u.
// Only root paths are ever painted, so each colour is a vertical chain and
// a vertex shares its colour with at most one child, `same[v]`. Let
// f[u] = colours on root..u, kept by preorder position under range add and
// range max. Painting root..x points `same` down the path; each pointer
// that moves adds one to the subtree it leaves and takes one from the
// subtree it enters. On a heavy chain the pointers that are not already on
// the heavy child are kept in a second tree, so a paint only visits the
// ones it has to move (O(log n) amortised, the access bound of link-cut
// trees).
// original: corpus/submissions/5463990.cpp

let (n, q): (1..=1e5, 1..=1e5) = read()
domain V = 1..=n
domain E = 0..(n - 1)
let edges: [E](u: V, v: V) = read()
let g = Graph<V, E>.undirected(edges)
let t = Rooted(u => g.out(u), V.first)

var f = SegTree<t.Pre, Int, Int>(max, act: add, a: [t.Pre] p => t.depth(t.at[p]) + 1)
var same: [V]?V = [V] _ => None
// 1 where same[v] is not the heavy child
var off = SegTree<t.Pre, Int>(+, [t.Pre] p => if t.heavy[t.at[p]] == none: 0 else: 1)

fn point(v: V, to: ?V):
    if same[v] == to: return
    if let c = same[v]: f.apply(t.sub(c), 1)
    if let c = to: f.apply(t.sub(c), -1)
    same[v] = to
    off.set(t.tin[v], if to == t.heavy[v]: 0 else: 1)

fn paint(x: V):
    var below: ?V = None
    for (range, rising) in t.chains(x, V.first, false):
        let bottom = t.at[range.end.left]
        point(bottom, below)
        // every other vertex of this stretch continues to its heavy child
        var b = off.max_right(range.start, s => s == 0)
        while b.ord < range.end.ord - 1:
            let v = t.at[b.right]
            point(v, t.heavy[v])
            b = off.max_right(b, s => s == 0)
        below = t.at[range.start.right]

for _ in 0..q:
    let kind: 1..=3 = read()
    if kind == 1:
        let x: V = read()
        paint(x)
    elif kind == 2:
        let (x, y): (V, V) = read()
        let top = t.lca(x, y)
        print(f.get(t.tin[x]) + f.get(t.tin[y]) - 2 * f.get(t.tin[top]) + 1)
    else:
        let x: V = read()
        print(f.fold(t.sub(x)))

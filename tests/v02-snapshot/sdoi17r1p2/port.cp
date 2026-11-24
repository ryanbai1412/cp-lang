// Source: SDOI 2017 Round 1 P2 (Tree Colouring), https://dmoj.ca/problem/sdoi17r1p2
// Problem: a tree on n <= 1e5 vertices rooted at 1, every vertex its own colour; m <= 1e5 ops:
//   "1 x" paints the path root..x a brand-new colour, "2 x y" prints the number of colours on
//   path x..y, "3 x" prints the most colours on a path from the root to a vertex in sub(x).
// Solution: only root paths are painted, so each colour is a vertical chain and a vertex shares
//   its colour with at most one child, same[v]. f[u] = colours on root..u lives in a range add,
//   range max SegTree over preorder; a pointer moving off child c adds 1 to sub(c), onto c takes
//   1. Painting walks the heavy chains of root..x and re-points each; a second SegTree marks the
//   vertices not pointing at their heavy child so only those are visited (amortised O(log n),
//   the link-cut access bound), each an O(log n) update. Query 2 is f[x] + f[y] - 2 f[lca] + 1.
// original: corpus/submissions/5463990.cpp

let (n, q): (1..=1e5, 1..=1e5) = read()
domain V = 1..=n
domain E = 0..(n - 1)
let edges: [E](u: V, v: V) = read()
let g = Graph<V, E>.undirected(edges)
let t = Rooted(u => g.out(u), V.first!)

var f = SegTree<t.Pre, Int, Int>(max, act: add, a: [t.Pre] p => t.depth(t.at[p]) + 1)
var same: [V]?V = [V] _ => None
// 1 where same[v] is not the heavy child
var off = SegTree<t.Pre, Int>(+, [t.Pre] p => if t.heavy[t.at[p]] == None: 0 else: 1)

fn point(v: V, to: ?V):
    if same[v] == to: return
    if let c = same[v]: f.apply(t.sub(c), 1)
    if let c = to: f.apply(t.sub(c), -1)
    same[v] = to
    off.set(t.tin[v], if to == t.heavy[v]: 0 else: 1)

fn paint(x: V):
    var below: ?V = None
    for (range, rising) in t.chains(x, V.first!, false):
        let (lo, hi) = range.ends
        let bottom = t.at[t.Pre.nth(hi - 1)]
        point(bottom, below)
        // every other vertex of this stretch continues to its heavy child
        var b = off.right_while(t.Pre.nth(lo).l, s => s == 0)
        while b.ord < hi - 1:
            let v = t.at[b.right!]
            point(v, t.heavy[v])
            b = off.right_while(b, s => s == 0)
        below = t.at[t.Pre.nth(lo)]

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

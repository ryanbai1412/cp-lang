// Baltic OI '17 P2 - Toll (DMOJ: btoi17p2, 20pt, TL 1.0s, ML 1024MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Directed weighted graph on n vertices 0..n-1 arranged in layers of size k <= 5:
// every edge (a, b, t) satisfies b / k == a / k + 1. Answer o shortest-path
// queries (a, b), printing -1 if unreachable.
//
// Each layer transition g -> g + 1 is a k x k min-plus matrix over ?Int
// (none = unreachable). Folding a SegTree of these non-commutative matrices
// over layer gaps answers any query in O(k^3 log(n / k)).
use std.segtree.{SegTree}
use std.algebra.{Monoid}
use std.graph.{Graph, dijkstra}

let (k, n, m, o): (1..=5, 1..=5e4, 0..=25e4, 1..=1e4) = read()
domain V = 0..n
domain E = 0..m
domain Q = 0..o
domain Slot = 0..k
// WISH W5: non-typeable layered-DAG edge constraint (b / k == a / k + 1) and no-duplicate-edge rule
let edges: [E](a: V, b: V, t: 0..=1e4) = read() where all(e.b.to_idx / k == e.a.to_idx / k + 1 for e in edges)
let queries: [Q](a: V, b: V) = read()

let layers = (n + k - 1) / k
domain Gap = 0..max(1, layers - 1)
type Mat = [Slot, Slot]?Int

fn mat_id() -> Mat:
    // WISH W6: 2D array comprehension over product domain `[Slot, Slot] (i, j) => ...`
    [Slot, Slot] (i, j) => if i == j: 0 else: none

fn mat_mul(x: Mat, y: Mat) -> Mat:
    var out: Mat = none
    for i in Slot:
        for mid in Slot:
            if let d1 = x[i, mid]:
                for j in Slot:
                    if let d2 = y[mid, j]:
                        out[i, j] min= d1 + d2
    out

const MinPlus = Monoid<Mat>(id: mat_id(), op: (x, y) => mat_mul(x, y))

fn solve() -> [Q]Int:
    var step = [Gap] _ => ([Slot, Slot] (_, _) => none)
    for e in E:
        let (a, b, t) = (edges[e].a.to_idx, edges[e].b.to_idx, edges[e].t)
        let g = Gap(a / k)
        step[g][Slot(a % k), Slot(b % k)] min= t
    let st = SegTree<Gap, Mat>.from(MinPlus, step)
    var out: [Q]Int = -1
    for q in Q:
        let (a, b) = (queries[q].a.to_idx, queries[q].b.to_idx)
        let (ga, sa) = (a / k, Slot(a % k))
        let (gb, sb) = (b / k, Slot(b % k))
        if ga == gb:
            out[q] = if a == b: 0 else: -1
        elif ga < gb:
            let prod = st.fold(Gap(ga)..Gap(gb))
            out[q] = prod[sa, sb] ?? -1
    out

@brute(solve)
fn solve_brute() -> [Q]Int
    where n <= 200 and o <= 200:
    let g = Graph<V, E>.directed([E] e => (u: edges[e].a, v: edges[e].b))
    var out: [Q]Int = -1
    for q in Q:
        let dist = dijkstra::<V>(u => g.out(u), [(queries[q].a, 0)], arc => edges[arc.e].t)
        out[q] = dist[queries[q].b] ?? -1
    out

for ans in solve(): print(ans)

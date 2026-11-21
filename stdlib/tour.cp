//! A short tour: the examples from README.md, kept here so they stay runnable.
use std.algebra.{Monoid}
use std.dsu.{Dsu}
use std.fenwick.{Fenwick}
use std.segtree.{SegTree}
use std.lazy.{LazySegTree, AddMax}
use std.sparse.{SparseTable}
use std.graph.{Graph, dijkstra, scc}
use std.flow.{FlowGraph}
use std.tree.{Rooted, Centroids}
use std.mod.{Mod, Comb}
use std.strings.{z_function, suffix_array, find_all}
use std.geometry.{
    Pt,
    Vec,
    area2,
    closest_pair2,
    convex_hull,
    diameter2,
    locate,
    locate_convex,
    reflect,
    close,
}
use std.bitset.{Bitset}
use std.dp.{LiChao, Line}
use std.seq.{bsearch_real, ternary_max}
use std.spec.{poly_degree, poly_at, guess_at, dnc_break, convexity_break}

@test
fn one_tree_many_monoids():
    domain I = 1..=5
    let a = [I] i => [3, 1, 4, 1, 5][i.to_idx]
    let sums = SegTree<I, Int>(+, a)
    let highs = SegTree<I, Int>(max, a)
    assert sums.fold(I(2)..=I(4)) == 6
    assert highs.fold(I(2)..=I(4)) == 4
    // your own monoid is one line: (best, count of best)
    let best = Monoid<(Int, Int)>(
        (Int.MIN, 0),
        (x, y) => if x.0 > y.0: x else: if y.0 > x.0: y else: (x.0, x.1 + y.1),
    )
    let t = SegTree<I, (Int, Int)>(best, [I] i => (a[i], 1))
    assert t.fold(I(1)..=I(4)) == (4, 1) and t.fold(I(1)..=I(2)) == (3, 1) and t.all == (5, 1)

@test
fn indices_know_their_container():
    domain V = 1..=4
    domain E = 0..3
    var d = Dsu<V>()
    let edges = [E] e => [(u: V(1), v: V(2)), (u: V(3), v: V(4)), (u: V(2), v: V(1))][e.to_idx]
    var joined = 0
    for e in E:
        if d.unite(edges[e].u, edges[e].v): joined += 1
    assert joined == 2 and d.components == 2 and d.same(V(1), V(2)) and not d.same(V(2), V(3))
    // `d.find(e)` for an edge index `e` is rejected: an E is not a V

@test
fn ranges_and_boundaries():
    domain I = 0..6
    var f = Fenwick<I, Int>()
    for i in I: f.add(i, i.to_idx + 1)
    assert f.sum(I(1)..=I(3)) == 2 + 3 + 4
    assert f.sum(I(1)..I(3)) == 2 + 3
    assert f.prefix(I(3).l) == 1 + 2 + 3 and f.prefix(I(3).r) == 1 + 2 + 3 + 4
    var lz = LazySegTree<I, Int>(AddMax, [I] i => 0)
    lz.apply(I(2)..=I(4), 10)
    lz.apply(I(4)..=I(5), 1)
    assert lz.fold(I(0)..=I(5)) == 11 and lz.fold(I(0)..=I(1)) == 0
    let rmq = SparseTable<I, Int>(min, [I] i => [5, 2, 7, 1, 8, 3][i.to_idx])
    assert rmq.fold(I(0)..=I(2)) == 2 and rmq.fold(I(4)..=I(5)) == 3 and rmq.fold(I(3)..=I(3)) == 1

@test
fn graphs_are_functions():
    domain V = 1..=4
    domain E = 0..4
    let ends = [E] e => [
        (u: V(1), v: V(2)),
        (u: V(2), v: V(3)),
        (u: V(3), v: V(1)),
        (u: V(3), v: V(4)),
    ][e.to_idx]
    let w = [E] e => [5, 1, 1, 7][e.to_idx]
    let g = Graph<V, E>.directed(ends)
    let dist = dijkstra(g, from: [(V(1), 0)], w: a => w[a.e])
    assert [dist[v] for v in V] == [0, 5, 6, 13]
    let (C, comp) = scc(g)
    assert C.K == 2 and comp[V(1)] == comp[V(3)] and comp[V(4)] != comp[V(1)]
    // an implicit graph needs no Graph at all: a grid walk, a number line, a state space
    domain N = 0..10
    let hops = dijkstra(
        from: [(N(1), 0)],
        next: u => [N(x) for x in [u.to_idx * 2, u.to_idx + 1] if x < 10],
        w: a => 1,
    )
    assert hops[N(9)] == 4

@test
fn flow_and_cut():
    domain V = 0..4
    var net = FlowGraph<V>()
    let a = net.add(V(0), V(1), 3)
    net.add(V(0), V(2), 2)
    net.add(V(1), V(2), 5)
    net.add(V(1), V(3), 2)
    net.add(V(2), V(3), 3)
    assert net.max_flow(V(0), V(3)) == 5
    assert net.flow(a) == 3
    assert net.min_cut(V(0))[V(3)] == false

@test
fn paths_on_trees():
    domain V = 1..=6
    let up = [V] v => [1, 1, 1, 2, 2, 5][v.to_idx]
    var kids = [V] v => List<V>()
    for v in V:
        if v != V(1): kids[V(up[v])].push(v)
    let r = Rooted(root: V(1), adj: u => kids[u])
    assert r.lca(V(4), V(6)) == V(2) and r.dist(V(4), V(6)) == 3 and r.dist(V(3), V(6)) == 4
    // a value per vertex lives in any structure over the tree's own positions
    var st = SegTree(+, r.reorder([V] v => v.to_idx + 1))
    assert r.fold_path(V(4), V(6), st) == 4 + 2 + 5 + 6
    assert st.fold(r.sub(V(2))) == 2 + 4 + 5 + 6
    st.set(r.pos(V(2)), 100)
    assert r.fold_path(V(4), V(3), st) == 4 + 100 + 1 + 3

@test
fn every_root_at_once():
    domain V = 1..=5
    let g = Graph.undirected([(V(1), V(2)), (V(2), V(3)), (V(2), V(4)), (V(4), V(5))])
    let t = Rooted(g, V(1))
    // the height DP from one root, then from every root with the same three arguments
    let h1 = t.up(max, (h, c, p) => h + 1, (h, v) => max(h, 0))
    let all = t.reroot(max, (h, c, p) => h + 1, (h, v) => max(h, 0))
    assert [h1[v] for v in V] == [3, 2, 0, 1, 0] and [all[v] for v in V] == [3, 2, 3, 2, 3]
    // centroid ancestors split every path: the nearest common one lies on it
    let cd = Centroids<V>(g)
    assert cd.parent(V(2)) == none and [x.c for x in cd.ancestors(V(5))] == [V(5), V(4), V(2)]

@test
fn numbers_modulo_a_prime():
    type M = Mod<998244353>
    assert M(2).pow(10) == M(1024) and M(3) / M(3) == M(1) and M(-1) + M(1) == M(0)
    let c = Comb<M>()
    assert c.choose(10, 3) == M(120)
    // a mod number is a number: the generic structures take it as they are
    domain I = 0..4
    var f = Fenwick<I, M>()
    f.add(I(0), M(998244352))
    f.add(I(1), M(5))
    assert f.sum(I(0)..=I(3)) == M(4)

@test
fn strings_and_points():
    assert z_function("abacaba") == [7, 0, 1, 0, 3, 0, 1]
    assert suffix_array("abracadabra") == [10, 7, 0, 3, 5, 8, 1, 4, 6, 9, 2]
    assert [i for i in find_all("abracadabra", "abra")] == [0, 7]
    let square = [Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]
    assert area2(square) == 8 and locate(square, Pt(1, 1)) == 1 and locate(square, Pt(3, 1)) == -1
    assert locate_convex(square, Pt(1, 1)) == 1 and diameter2(square) == 8
    assert convex_hull([Pt(1, 1), Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]) == square
    assert closest_pair2(square) == 4
    let r = reflect(Vec(1.0, 3.0), Vec(0.0, 0.0), Vec(4.0, 0.0))
    assert close(r.x, 1.0) and close(r.y, -3.0)

@test
fn sets_of_indices():
    domain D = 0..100
    var seen = Bitset<D>()
    for x in [3, 64, 99]: seen.add(D(x))
    assert seen.len == 3 and seen.contains(D(64)) and not seen.contains(D(65))
    assert [d.to_idx for d in seen.each()] == [3, 64, 99]
    // subset sums: shift and union, 64 at a time
    var reach = Bitset<D>()
    reach.add(D(0))
    for w in [5, 11, 40]: reach.union(reach.shifted(w))
    assert reach.contains(D(56)) and reach.contains(D(16)) and not reach.contains(D(17))

@test
fn ask_the_brute_force_what_it_is():
    // sums of squares, from the slow definition: which polynomial is it?
    let slow = [sum(k * k for k in 1..=n) for n in 0..8]
    assert poly_degree(slow) == 3
    assert poly_at(slow, 1000) == 1000 * 1001 * 2001 / 6
    // tilings of a 2 x n strip: which recurrence is it?
    type M = Mod<1000000007>
    assert guess_at([M(1), M(1), M(2), M(3), M(5), M(8), M(13), M(21)], 50) == M(20365011074)
    // is this DP divide-and-conquer friendly? is this answer convex in k?
    assert dnc_break((i, j) => (i - j) * (i - j) + j, 12, 12) == none
    assert convexity_break([9, 4, 1, 0, 1, 4]) == none and convexity_break([0, 5, 6, 9]) == 0

@test
fn a_fold_on_a_missing_map_key_starts_from_the_identity():
    var m = Map()
    m[3] max= -5
    m[5] min= 5
    m[5] min= 2
    m[6] += 2
    m[7] *= 3
    m[7] *= 3
    assert m[3] == -5 and m[5] == 2 and m[6] == 2 and m[7] == 9
    assert m.remove(5) == 2 and m.remove(5) == none

@test
fn a_recurrence_calls_its_optimization_by_name():
    let (h, c) = ([1, 2, 3, 4, 5], 6)
    var hull = LiChao<Line, Int>(min, 0..=10 ** 6)
    var dp = [0 for _ in h]
    for i in 0..h.len:
        if i > 0: dp[i] = hull.at(h[i])! + h[i] * h[i] + c
        hull.add(Line(-2 * h[i], dp[i] + h[i] * h[i]))
    assert dp.last! == 20
    let root2 = bsearch_real(0.0, 2.0, x => x * x >= 2.0)!
    assert abs(root2 * root2 - 2.0) < 1e-9
    let a = [1, 4, 6, 5, 2]
    assert ternary_max(0..a.len, i => a[i]) == 2

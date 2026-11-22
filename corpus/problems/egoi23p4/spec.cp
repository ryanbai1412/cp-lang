// egoi23p4: rebuild a weighted graph from its all-pairs widest-path widths (bikes) and its
// all-pairs narrowest-bottleneck widths (cars, given as k minus the width). Both matrices must
// already be closed under their path algebra; an edge u-v may carry only weights between the two
// bottlenecks, so try both ends, check the closures, and print the union of a minimum and a
// maximum spanning forest of the candidates.
let (n, k) = read<(1..=500, 1..=1_000_000_000)>()
domain V = [n]
domain Pair = (V, V)
fn read_triangle(flip: Bool) -> [V, V]?Int:
    var m: [V, V]?Int = none
    for i in V:
        for j in ..i:
            let x = read<0..=1_000_000_000>()
            let w = if flip: k - x else: x
            (m[i, j], m[j, i]) = (some(w), some(w))
    m
let narrow = read_triangle(true)
let wide = read_triangle(false)
let widest = Semiring(max, min)
let narrowest = Semiring(min, max)
fn closes_to(d: [V, V]?Int, over: Semiring, want: [V, V]?Int) -> Bool:
    var c = d.clone()
    floyd_warshall(mut c, over: over)
    Pair.all(p => p.0 == p.1 or c[p] == want[p])
fn fits(p: Pair) -> Bool = (wide[p] ?? -1) >= (narrow[p] ?? inf)
let try_wide: [V, V]?Int = [Pair] p => if fits(p): wide[p] else: none
let try_narrow: [V, V]?Int = [Pair] p => if fits(p): narrow[p] else: none
if not (closes_to(wide, widest, wide) and closes_to(narrow, narrowest, narrow)
        and closes_to(try_wide, widest, wide) and closes_to(try_narrow, narrowest, narrow)):
    print("NO")
else:
    var edges: List<(u: V, v: V, w: Int)> = []
    for (u, v) in Pair if u > v and fits((u, v)):
        edges.push((u: u, v: v, w: wide[u, v]!))
        edges.push((u: u, v: v, w: narrow[u, v]!))
    let low = kruskal(edges).take
    let high = kruskal(edges.map(e => (u: e.u, v: e.v, w: -e.w)).collect()).take
    let keep = edges.keys.filter(e => low[e] or high[e]).collect()
    print(keep.len)
    for e in keep:
        print(edges[e].u, edges[e].v, edges[e].w)

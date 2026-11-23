// egoi23p4: rebuild a weighted graph from its all-pairs widest-path widths (bikes) and its
// all-pairs narrowest-bottleneck widths (cars, given as k minus the width). Both matrices must
// already be closed under their path algebra; an edge u-v may carry only weights between the two
// bottlenecks, so try both ends, check the closures, and print the union of a minimum and a
// maximum spanning forest of the candidates.
let (n, k) = read<(1..=500, 1..=1_000_000_000)>()
domain V = [n]
let narrow = read<[Pair<V>]0..=1_000_000_000>().map(x => Some(k - x)).symmetric(diag: None)
let wide = read<[Pair<V>]0..=1_000_000_000>().map(Some).symmetric(diag: None)
let widest = Semiring(max, min)
let narrowest = Semiring(min, max)
fn fits(u: V, v: V) -> Bool = (wide[u, v] ?? -1) >= (narrow[u, v] ?? inf)
fn only_fits(d: [V, V]?Int) -> [V, V]?Int = [V, V] (u, v) => if u != v and fits(u, v): d[
    u,
    v,
] else: None
fn closes_to(d: [V, V]?Int, over: Semiring, want: [V, V]?Int) -> Bool:
    let c = d.closure(over: over)!
    (V, V).all((u, v) => u == v or c[u, v] == want[u, v])
if not (
    closes_to(wide, widest, wide) and closes_to(narrow, narrowest, narrow) and closes_to(
        only_fits(wide),
        widest,
        wide,
    ) and closes_to(
        only_fits(narrow),
        narrowest,
        narrow,
    )
):
    print("NO")
else:
    let edges = [
        (u: u, v: v, w: w!) for (u, v) in V.pairs() if fits(u, v) for w in [
            wide[u, v],
            narrow[u, v],
        ]
    ]
    let low = kruskal(edges).take
    let high = kruskal(edges, by: e => desc(e.w)).take
    let keep = edges.keys.filter(e => low[e] or high[e]).collect()
    print(keep.len)
    for e in keep:
        print(edges[e].u, edges[e].v, edges[e].w)

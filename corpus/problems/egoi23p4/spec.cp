// egoi23p4: rebuild a weighted graph from its all-pairs widest-path widths (bikes) and its
// all-pairs narrowest-bottleneck widths (cars, given as k minus the width). Both matrices must
// already be closed under their path algebra; an edge u-v may carry only weights between the two
// bottlenecks, so try both ends, check the closures, and print the union of a minimum and a
// maximum spanning forest of the candidates.
let (n, k) = read<(1..=500, 1..=1_000_000_000)>()
domain V = [n]
fn read_triangle(flip: Bool) -> [V, V]?Int:
    var m: [V, V]?Int = None
    for i in V:
        for j in ..i:
            let x = read<0..=1_000_000_000>()
            let w = if flip: k - x else: x
            (m[i, j], m[j, i]) = (Some(w), Some(w))
    m
let narrow = read_triangle(true)
let wide = read_triangle(false)
fn closes_to(d: [V, V]?Int, over: Semiring, want: [V, V]?Int) -> Bool:
    var c = d.clone()
    floyd_warshall(mut c, over: over)
    V.all(u => V.all(v => u == v or c[u, v] == want[u, v]))
var try_wide: [V, V]?Int = None
var try_narrow: [V, V]?Int = None
var edges: List<(u: V, v: V, w: Int)> = []
for u in V:
    for v in ..u:
        if (wide[u, v] ?? -1) >= (narrow[u, v] ?? inf):
            (try_wide[u, v], try_wide[v, u]) = (wide[u, v], wide[u, v])
            (try_narrow[u, v], try_narrow[v, u]) = (narrow[u, v], narrow[u, v])
            edges.push((u: u, v: v, w: wide[u, v]!))
            edges.push((u: u, v: v, w: narrow[u, v]!))
let widest = Semiring(max, min)
let narrowest = Semiring(min, max)
if not (closes_to(wide, widest, wide) and closes_to(narrow, narrowest, narrow)
        and closes_to(try_wide, widest, wide) and closes_to(try_narrow, narrowest, narrow)):
    print("NO")
else:
    let low = kruskal(edges).take
    let high = kruskal(edges, by: e => desc(e.w)).take
    let keep = edges.keys.filter(e => low[e] or high[e]).collect()
    print(keep.len)
    for e in keep:
        print(edges[e].u, edges[e].v, edges[e].w)

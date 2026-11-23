// A range element type binds a generic used as a domain (R04.4.30): `R` is `0..2`, then `1..=3`.
fn hits<L, R>(adj: [L]List<R>) -> [R]Int:
    var out: [R]Int = 0
    for l in L:
        for r in adj[l]: out[r] += 1
    out
let adj: [0..2]List<0..2> = [[0, 1], [0]]
print(hits(adj))
let ups: [0..3]List<1..=3> = [[3, 1], [3], [2, 3]]
let h = hits(ups)
print(h, h[3])

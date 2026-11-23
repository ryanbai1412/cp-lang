// expect: ok, output "2 1"
// rule: 04.4.30
// A range type in element position binds a generic parameter that the generic uses as a
// domain, as `0..2` does in `[0..2]List<0..2>`: `R` indexes the result.
fn hits<L, R>(adj: [L]List<R>) -> [R]Int:
    var out: [R]Int = 0
    for l in L:
        for r in adj[l]: out[r] += 1
    out
let adj: [0..2]List<0..2> = [[0, 1], [0]]
print(hits(adj))

let (n, m): (1..=2e5, 0..=2e5) = read()
domain V = 1..=n
var out: [V]List<V> = []
for _ in 0..m:
    let (u, v): (V, V) = read()
    out[u].push(v)

@memo
fn paths(v: V) -> Mod<1_000_000_007>:
    if v == V(n):
        return 1
    var total: Mod<1_000_000_007> = 0
    for w in out[v]:
        total += paths(w)
    total

print(paths(V(1)))

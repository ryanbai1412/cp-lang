// expect: L0517
domain V = 0..4
var par: [V]V = [V] v => v
fn find(v: V) -> V:
    var x = v
    while par[x] != x: // <- here
        par[x] = par[par[x]]
        x = par[x]
    x
print(find(V(0)))

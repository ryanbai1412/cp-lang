use std.dsu.{Dsu}
use std.mod.{Mod, Comb}

const P = 1_000_000_007
let (n, m): (1..=100, 0..=100) = read()
domain V = 1..=n

var d = Dsu<V>()
for _ in 0..m:
    let (u, v): (V, V) = read()
    d.unite(u, v)

let c = d.components
let cb = Comb<P>(n)
var ways: Mod<P> = cb.choose(n, c)
print(c, ways)

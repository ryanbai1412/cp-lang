// error: `.push(...)` would keep the lent slice
let n: 2..=9 = read()
domain V = 1..=n
fn keep(mut parts: List<[V]Int>, a: [V]Int, l: V, r: V):
    parts.push(a[l..=r])
var parts: List<[V]Int> = []
let a: [V]Int = read()
keep(mut parts, a, V(1), V(2))

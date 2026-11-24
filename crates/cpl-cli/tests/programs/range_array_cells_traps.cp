// A cell of `[V]0..=V.count` reads at that range: `min(dep[u] + 1, V.count)`
// and the copy into `c` store unchecked, while `dep[u] + k` can leave the
// range and traps in both engines.
fn walk(n: 1..=100, k: Int) -> Int:
    domain V = 0..n
    var dep: [V]0..=V.count = 0
    var c: 0..=V.count = 0
    for v in V:
        let u = V.first!
        dep[v] = min(dep[u] + 1, V.count)
        c = dep[v]
    print(c, dep[V.last!])
    let u = V.first!
    dep[u] = dep[u] + k
    dep[u]
let n: 1..=100 = read()
print(walk(n, 2))
print(walk(n, n))

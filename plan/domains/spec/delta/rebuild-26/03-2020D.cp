// 2020D Connect the Dots: per step d, a difference array over V marks which
// x are joined to x + d; unite those in a Dsu and count roots.
@!multitest
let (n, m): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
domain Step = 1..=10
let ops = read<[m](a: V, d: Step, k: 0..=n)>()
var link: [Step, V]Int = 0
for (a, d, k) in ops:
    if k > 0:
        link[d, a] += 1
        link[d, V(Int(a) + k * Int(d))] -= 1
var dsu = Dsu<V>.new()
for d in Step:
    for x in V:
        if Int(x) > Int(d): link[d, x] += link[d, V(Int(x) - Int(d))]
        if link[d, x] > 0: dsu.unite(x, V(Int(x) + Int(d)))
print(dsu.roots().count())

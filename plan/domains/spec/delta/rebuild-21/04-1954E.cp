// CF 1954E Chain Reaction: answer(k) = sum over j of the number of maximal runs with
// a > j * k. runs(t) for every threshold t comes from a Dsu, adding monsters tallest first.
let n = read<1..=1e5>()
domain V = 0..n
let a = read<[V]1..=1e5>()
let top = a.max()!

var by_health = [1..=top] _ => List<V>()
for v in V: by_health[a[v]].push(v)

var dsu = Dsu<V>()
var alive: [V]Bool = false
var runs: [0..top]Int = 0
var cur = 0
for t in (0..top).rev:
    for v in by_health[t + 1]:
        alive[v] = true
        cur += 1
        for w in [v.prev, v.next]:
            if let u = w:
                if alive[u] and dsu.unite(u, v):
                    cur -= 1
    runs[t] = cur
print([(0..top).step(k).map(t => runs[t]).sum() for k in 1..=top])

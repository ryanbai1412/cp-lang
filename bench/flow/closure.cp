// Maximum closure: 50000 vertices with profits in [-10^6, 10^6], 300000 random needs.

domain V = 0..50000
var seed = 1
fn next(n: Int) -> Int:
    seed = seed * 48271 % 2147483647
    seed % n

var profit: [V]Int = 0
for v in V: profit[v] = next(2000001) - 1000000
// `u` needs `v`: taking `u` forces taking `v`, an arc `v -> u` of `before`.
var before: [V]List<V> = []
for _ in 0..300000:
    let u = next(V.count)
    let v = next(V.count)
    before[V(v)].push(V(u))
let (value, take) = max_closure(before, profit: v => profit[v])
print(value, [v for v in V if take[v]].len)

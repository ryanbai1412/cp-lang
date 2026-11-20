// Names of `stdlib/prelude.cp` need no `use`: types and fns alike.
let n: 1..=10 = read()
domain V = 0..n
var d = Dsu<V>()
d.unite(V.first, V.last)
print(gcd(12, 18), lcm(4, 6), d.same(V.first, V.last))
var h = MinHeap<Int>()
h.push(3)
h.push(1)
print(h.pop()!)

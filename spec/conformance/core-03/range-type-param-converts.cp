// expect: ok, output "1 0"
// rule: 03.1.4
// A type parameter bound to a range converts like `Int(x)`: `Counter<K, 0..=10>()` takes `V(0)` as its default.
domain K = 0..5
var c = Counter<K, 0..=10>()
c.add(K(1), 1)
print(c[K(1)], c[K(2)])

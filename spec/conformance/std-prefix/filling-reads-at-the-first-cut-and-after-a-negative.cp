// expect: ok, output "0 5\n-2 -2 5"
// rule: R07.1.6
// A filling table reads 0 at the first cut, and a negative cell lowers every later fold but not an earlier one.
domain V = 1..=3
var f = Prefix<V, Int>.filling()
f.set(1, 5)
print(f[V(1).l], f[V(1)])
f.set(2, -7)
print(f[V(2)], f[V(2).r], f[V(1).r])

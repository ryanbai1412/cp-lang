// expect: ok, output "5 12 12"
// rule: R07.1.6
// A table filled one cell at a time reads like a built `Prefix`: `f[v]` folds through `v`, `f[c]` folds left of cut `c`.
domain V = 1..=3
var f = Prefix<V, Int>.filling()
f.set(1, 5)
f.set(2, 7)
print(f[V(1)], f[V(2)], f[V(2).r])

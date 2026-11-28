// expect: ok, output "8\n5\n3\n8"
// rule: 06.4.16
// rule: std.subset.Subset
// `V.subsets` is the domain std declares with `@member(subsets)` above
// `pub struct Subset<B>`: a `[V.subsets]T` table is indexed by a `Subset<V>`,
// and walking the domain visits every subset, each with `Subset`'s methods.
domain V = 0..3
var best: [V.subsets]Int = 0
let s = Subset<V>.empty.with(V(0)).with(V(2))
best[s] = 5
var n = 0
for t in V.subsets: n += 1
print(n)
print(best[s])
print(s.items.collect().len + 1)
var m = 0
for t in V.subsets:
    if t.contains(V(0)): m += t.without(V(1)).len + Int(t.contains(V(1)))
print(m)

// expect: ok, output "2 3 9 2\n10"
// rule: R04.2.15 R04.2.18
// `v.ord` is `Domain`'s getter: a declared domain's position counts from its start, an `Int` is its
// own ordinal, and a parameter bounded by `Domain` takes the instance's.
domain V = 3..7
fn o<D: Domain>(x: D) -> Int = x.ord
print(V(5).ord, o(V(6)), o(9), (2).ord)
let a: [V]Int = [V] v => v.ord * 10
print(a[V(4)])

// expect: ok, output "20 0 30"
// `a.get(i)` on a domain-keyed `[V]T` reads by zero-based `Int` position, as on every `Indexed`; past the end it is `None`.
domain V = 1..=3
let a: [V]Int = [V] v => 10 * Int(v)
print(a.get(1) ?? 0, a.get(3) ?? 0, a.get(V.last!.ord) ?? 0)

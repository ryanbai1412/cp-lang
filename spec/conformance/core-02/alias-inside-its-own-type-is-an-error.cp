// expect: E0323
// rule: 02.6.2
// No import or std type is named `A`, so `List<A>` would need `A` itself.
type A = List<A> // <- here
let x: A = []
print(x.len)

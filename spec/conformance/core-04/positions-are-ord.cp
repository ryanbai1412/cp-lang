// expect: ok, output "3 YES"
// rule: 04.2.23
// Positions are `Ord` in domain order, so a `SortedSet<V>` holds them.
domain V = 1..=5
let s = SortedSet.of([v for v in V if Int(v) % 2 == 1])
print(s.len, yes(V.first! < V.last!))

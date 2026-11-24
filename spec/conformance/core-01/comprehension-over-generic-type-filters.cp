// expect: ok, output "3"
// rule: 01.8.1
// A comprehension's `in` takes a generic type as its sequence, and its `if` still filters: `Pair<V> if` is not a comparison.
domain V = 0..4
print([p for p in Pair<V> if p.lo.ord == 0].len)

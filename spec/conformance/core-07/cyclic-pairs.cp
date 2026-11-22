// expect: ok, output "1"
// rule: 07.7.8
print([1, 2, 3].cyclic_pairs.count((x, y) => x > y))

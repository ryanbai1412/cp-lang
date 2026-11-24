// expect: ok, output "1 x 1 y 2 x 2 y\n4 0"
// rule: 07.7.12
// `product(a, b)` is in the prelude: every pair in lexicographic order, `a` outer, and nothing when either side is empty.
print([(x, y) for (x, y) in product([1, 2], "xy")], sep: " ")
print(product([1, 2], "xy").count(), product([0..0] i => i, [1]).count())

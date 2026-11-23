// expect: ok, output "4499999998500000000 3999999999999999999"
// rule: 07.9.6
// A range's own `sum()` is a closed form; an O(n) walk of 3e9 elements would time out.
print((0..3e9).sum(), (3..4e9).step(2).sum())

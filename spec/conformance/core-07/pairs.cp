// expect: ok, output "3"
// rule: 07.7.6
print([1, 2, 3].pairs().count((x, y) => x < y))

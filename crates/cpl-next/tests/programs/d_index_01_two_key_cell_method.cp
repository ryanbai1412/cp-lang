// expect: ok, output "2 1 0"
// rule: R12.4.6
// A writing method on a cell picked by two keys lends that one cell, as with one key.
domain A = 0..2
domain B = 0..2
var g: [A, B]List<Int> = []
g[A(0), B(1)].push(1)
g[A(0), B(1)].push(2)
g[A(1), B(0)].push(3)
print(g[A(0), B(1)].len, g[A(1), B(0)].len, g[A(1), B(1)].len)

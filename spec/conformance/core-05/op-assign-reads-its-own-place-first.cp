// expect: ok, output "abab xx xxx\n1 2 3\n2 3 2 3"
// rule: 01.6.4
// `p op= e` reads `e` before it writes `p`, so a `Str`, a boxed array cell or a list cell can be combined with itself or a neighbour and gets each value as it was.
var s = "ab"
s += s
var g: [0..2, 0..2]Str = "x"
g[1, 0] += g[1, 0]
g[0, 1] += g[1, 0]
var xs = [[1], [2, 3]]
xs[0] += xs[1]
xs[1] += xs[1]
print(s, g[1, 0], g[0, 1])
print(xs[0])
print(xs[1])

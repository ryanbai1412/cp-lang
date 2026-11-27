// expect: E0503
// rule: R01.8.5
// A write to a table's cell takes its keys in one pair of brackets too: `g[i][1] = v` is E0503,
// while an array of arrays and a list of lists still write `h[a][b]`.
domain V = 0..2
var h: [V][V]Int = [V] _ => [V] v => Int(v)
h[V(0)][V(1)] = 7
var xs = [[1, 2], [3, 4]]
xs[1][0] += 9
var g = [2, 2] (r, c) => r * 2 + c + 1
g[0, 1] = 8
print(h[V(0)][V(1)], xs[1][0], g[0, 1])
let i = 1
g[i][1] = 5  // <- here

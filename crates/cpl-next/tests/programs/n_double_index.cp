// expect: E0503
// rule: R01.8.5
// A table's cell takes all its keys in one pair of brackets: `t[a][b]` on `[(V, V)]T` is
// E0503, as `g[r][c]` on `[R, C]T` is. A row `g[r]`, its slice, a list of lists and an
// array of arrays still index twice.
let g = [2, 2] (r, c) => r * 2 + c + 1
print(g[0].len, (g[1])[0..2].len, g[1, 0])
let xs = [[1, 2], [3, 4]]
domain V = 0..2
let h: [V][V]Int = [V] _ => [V] v => Int(v)
print(xs[0][1], h[V(0)][V(1)])
let t: [(V, V)]Int = [(V, V)] (a, b) => Int(a) + Int(b)
print(t[V(1), V(0)])
print((t[V(1)])[V(1)])  // <- here

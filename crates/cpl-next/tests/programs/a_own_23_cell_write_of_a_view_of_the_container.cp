// expect: ok, output "1 2 1"
// rule: R04.1
// `r` views `g[1]`, and is written into another cell of `g`: the write copies it in, so the
// cell is its own list and later changes to `g[1]` don't reach it.
var g = [[1], [2]]
let r = g[1]
g[0] = r
g[1].push(5)
print(g[0][0] - 1, g[1].len, g[0].len)

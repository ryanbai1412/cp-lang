// expect: R0101
// `+=` on a cell of a 3-D `Str` array appends in place, and an index past the last dimension traps there, not in the next row.
var g: [0..2, 0..2, 0..3]Str = ""
g[1, 0, 2] += "ab"
g[1, 0, 2] += "c"
print(g[1, 0, 2], g[0, 0, 0].len)
let k = [3][0]
g[1, 1, k] = "x" // <- here
print(g[1, 1, 0])

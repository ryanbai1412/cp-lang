// expect: R0101 at 5 (checked)
// A column one past the end traps, even though the flat cell it would reach, the next row's first, exists.
// The array is top-level and holds Str, so the interpreter writes it through its global, non-Int store.
var g: [0..2, 0..3]Str = "."
let j = [3][0]
g[0, j] = "x"
print(g[1, 0])

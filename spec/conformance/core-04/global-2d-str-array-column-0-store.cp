// expect: ok, output "a b"
// Storing into column 0 of a top-level 2-D array of `Str` is in range: the first column is index 0, not past the start.
var g: [0..2, 0..2]Str = ""
g[1, 0] = "a"
g[0, 0] = "b"
print(g[1, 0], g[0, 0])

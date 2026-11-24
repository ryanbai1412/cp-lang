// expect: ok, output "4 3 2 1 edcba 20 10 0 1 4 3 2 5\nx, y, z p-q"
// `reverse` and `Str.join` are std bodies: lists, strings, tables and slices reverse in place.
use std.strings
var a = [1, 2, 3, 4]
a.reverse()
var s = "abcde"
s.reverse()
var g: [0..3]Int = [0..3] i => i * 10
g.reverse()
var b = [1, 2, 3, 4, 5]
b[1..4].reverse()
print(a, s, g, b)
print(", ".join(["x", "y", "z"]), "-".join("{c}" for c in "pq"))

use std.geometry.{ByAngle, Pt}
use std.math.choose

// Linear reach on List, [D]T, Str, ListView, StrView, Range.
let xs = [1, 2, 4, 7, 8, 12]
print(xs.reach(while: (a, b) => b - a <= 3))

domain D = 0..6
let arr: [D]Int = [D] i => xs[i.idx]
print(arr.reach(while: (a, b) => b - a <= 3))

let view = xs[1..5]
print(view.reach(while: (a, b) => b - a <= 3))

let s = "abcfgh"
print(s.reach(while: (a, b) => Int(b) - Int(a) <= 2))
print(s[1..5].reach(while: (a, b) => Int(b) - Int(a) <= 2))

print((0..5).reach(while: (a, b) => b - a <= 2))

// Empty and singleton (linear and cyclic).
let empty: List<Int> = []
print(empty.reach(while: (a, b) => true))
print(empty.reach(while: (a, b) => true, cyclic: true))
print([7].reach(while: (a, b) => true))
print([7].reach(while: (a, b) => true, cyclic: true))

// Cyclic cap k < len when always true.
print([1, 2, 3, 4].reach(while: (a, b) => true, cyclic: true))

// Regression: cf1284e angular half-turn count (dream/cf1284e.cp line 19).
let pts = [Pt(0, 0), Pt(4, 0), Pt(5, 4), Pt(1, 5), Pt(2, 2)]
var enclosed = 0
for c in 0..pts.len:
    let rays = [pts[v] - pts[c] for v in 0..pts.len if v != c].sorted(by: ByAngle)
    let ahead = rays.reach(while: (r, s) => r.cross(s) > 0, cyclic: true)
    enclosed += choose(pts.len - 1, 4) - sum(choose(k, 3) for k in ahead)
print(enclosed)

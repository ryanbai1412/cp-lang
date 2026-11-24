// expect: ok, output "2 1 2"
// Reading the parent `w.s` after `var t = w.s.xs` reads the field, so it is copied, not moved.
struct S:
    xs: List<Int>
    n: Int
struct W:
    s: S
let w = W(S([1], 2))
var t = w.s.xs
t.push(3)
let u = w.s
print(t.len, u.xs.len, w.s.n)

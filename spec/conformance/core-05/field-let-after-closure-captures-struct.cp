// expect: ok, output "2 1 1"
// A closure made before the `let` reads `s.xs` at each call, so the field is not moved out from under it.
struct S:
    xs: List<Int>
    n: Int
let s = S([5], 1)
let f = () => s.xs.len
var t = s.xs
t.push(6)
print(t.len, f(), f())

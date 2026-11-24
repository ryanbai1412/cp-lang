// expect: ok, output "4 7 3"
// Passing the whole struct after `var t = s.xs` reads the field again, so the `let` copies it.
struct S:
    xs: List<Int>
    n: Int
fn total(s: S) -> Int = s.xs.len + s.n
let s = S([1, 2, 3], 4)
var t = s.xs
t.push(0)
print(t.len, total(s), s.xs.len)

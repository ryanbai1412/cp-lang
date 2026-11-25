// expect: ok, output "1 2 3"
// rule: R02 tuples
// Tuple values: build, pass, return, destructure, swap.
fn swap(p: (Int, Int)) -> (Int, Int) = (p.1, p.0)
let (a, b) = swap((2, 1))
let t = (a, b, 3)
print(t.0, t.1, t.2)

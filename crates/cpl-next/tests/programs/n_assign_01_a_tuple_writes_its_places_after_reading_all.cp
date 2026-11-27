// expect: ok, output "3 2 1\n6\n4 5 1 2 3\n2 1"
// rule: R01.6.4
// `(p, q) = (e, f)` reads the whole right side before it writes any place, so it swaps.
var p = [1, 2, 3]
let l = 0
let r = 2
(p[l], p[r]) = (p[r], p[l])
print(p)
var (x, y) = (12, 18)
while y != 0: (x, y) = (y, x % y)
print(x)
var a = [1, 2, 3]
var b = [4, 5]
(a, b) = (b, a)
print(a, b)
fn pair() -> (Int, Int) = (1, 2)
var (u, v) = (0, 0)
(v, u) = pair()
print(u, v)

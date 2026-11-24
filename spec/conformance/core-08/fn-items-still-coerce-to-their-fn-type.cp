// expect: ok, output "5 2 7 2 5 5"
// rule: 08.1.10
// Each fn item has its own type, which coerces to its fn type wherever one is wanted: a `var` holding one item takes another, a list and an `if` mix them, and a fn-typed parameter takes any.
fn mx(a: Int, b: Int) -> Int = if a > b: a else: b
fn mn(a: Int, b: Int) -> Int = if a < b: a else: b
fn add(a: Int, b: Int) -> Int = a + b
fn apply(f: (Int, Int) -> Int, a: Int, b: Int) -> Int = f(a, b)
var g = mn
g = mx
let fs = [mx, mn, add]
let k = zero(3)
let h = if k == 0: mn else: mx
print(g(2, 5), fs[1](2, 5), fs[2](2, 5), h(2, 5), apply(mx, 2, 5), apply(g, 5, 2))
fn zero(n: Int) -> Int = if n == 0: 0 else: zero(n - 1)

// expect: ok, output "5 2 1"
// rule: 05.4.1
// `fold` runs only its `Int` op, never `f`, so it reads `t` in place; `g`
// has the op's type and writes `u`, so a fold of `u` must not see `g` run.
domain X = 0..5
var t = SegTree<X, Int>(max, fill: 0)
fn best(at: X) -> Int = t.fold(X.first!..=at)
fn add(at: X):
    t.set(at, best(at) + 1)
let f = (x: X) => add(x)
for x in X: f(x)
var u = SegTree<X, Int>(max, fill: 0)
fn bump(a: Int, b: Int) -> Int:
    u.set(X.first!, a)
    max(a, b)
let g = (a: Int, b: Int) => bump(a, b)
fn top(at: X) -> Int = u.fold(X.first!..=at)
print(t.fold(X.first!..=X.last!), g(1, 2), top(X.last!))

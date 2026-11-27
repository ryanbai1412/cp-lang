// expect: ok, output "4 6\n2 4\n-1 -2\n7"
// An operator on a struct is its trait's method; two impls of one trait differ by `Rhs`.
struct V: x: Int, y: Int
fn (a: V) + (b: V) -> V = V(a.x + b.x, a.y + b.y)
fn (a: V) * (k: Int) -> V = V(a.x * k, a.y * k)
fn -(a: V) -> V = V(-a.x, -a.y)
fn sum<T: Add>(a: T, b: T) -> T = a + b
let p = V(1, 2) + V(3, 4)
print(p.x, p.y)
let q = V(1, 2) * 2
print(q.x, q.y)
let n = -V(1, 2)
print(n.x, n.y)
print(sum(3, 4))

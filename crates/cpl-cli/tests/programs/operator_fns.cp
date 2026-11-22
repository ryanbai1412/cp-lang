struct P(x: Int, y: Int)

fn (a: P) + (b: P) -> P = P(a.x + b.x, a.y + b.y)
fn (a: P) - (b: P) -> P = P(a.x - b.x, a.y - b.y)
fn (a: P) * (b: P) -> P = P(a.x * b.x, a.y * b.y)
fn (a: P) * (k: Int) -> P = P(a.x * k, a.y * k)
fn -(a: P) -> P = P(-a.x, -a.y)
fn (a: P) < (b: P) -> Bool = a.x + a.y < b.x + b.y
fn (a: P) == (b: P) -> Bool = a.x == b.x and a.y == b.y

fn add_both<T: Add>(x: T, y: T) -> T = x + y
fn mul_both<T: Mul>(x: T, y: T) -> T = x * y
fn smaller<T: Ord>(x: T, y: T) -> T = if x < y: x else: y

let a = P(1, 2)
let b = P(3, 4)
let c = add_both(a, b)
let p = mul_both(a, b)
let d = (b - a) * 3
let e = -a
let m = smaller(P(5, 0), P(1, 1))
var pts = [P(10, 0), P(1, 2), P(3, 1)]
pts.sort()

print(c.x, c.y, p.x, p.y, d.x, d.y, e.x, e.y, m.x, m.y, pts[0].x, pts[2].x, P(1, 2) == a)

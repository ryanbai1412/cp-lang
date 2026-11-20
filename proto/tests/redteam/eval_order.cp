var c = 0
fn nx() -> Int:
    c += 1
    return c
fn sub(x: Int, y: Int) -> Int:
    return x - y
struct P:
    x: Int
    y: Int
    fn diff(self, k: Int, m: Int) -> Int:
        return (self.x - self.y) * 100 + k * 10 + m
var xs: List<Int> = List<Int>()
var a: [0..40, 0..40]Int = 0
print(sub(nx(), nx()))
print(nx() - nx())
let t = (nx(), nx())
print(t.0 - t.1)
let p = P(nx(), nx())
print(p.x - p.y)
print(p.diff(nx(), nx()))
print(min(nx() * 10, nx()))
a[nx(), nx()] = nx()
print(a[13, 14], a[14, 15], a[15, 14], a[14, 13], c)
print(nx(), nx())
print(nx() < nx())
print(max(nx(), nx() - 30))
xs.push(nx())
print(xs[0] - nx())
let q = P(x: nx(), y: nx())
print(q.x - q.y)
let u = [nx(), nx(), nx()]
print(u)

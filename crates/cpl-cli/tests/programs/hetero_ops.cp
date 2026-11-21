struct Vec:
    x: F64
    y: F64

impl Add for Vec:
    fn add(a, b) = Vec(a.x + b.x, a.y + b.y)

impl Sub for Vec:
    fn sub(a, b) = Vec(a.x - b.x, a.y - b.y)

impl Mul<F64> for Vec:
    fn mul(a, k) = Vec(a.x * k, a.y * k)

impl Div<F64> for Vec:
    fn div(a, k) = Vec(a.x / k, a.y / k)

struct Mat2:
    a: F64
    b: F64
    c: F64
    d: F64

impl Mul for Mat2:
    fn mul(x, y) = Mat2(
        x.a * y.a + x.b * y.c,
        x.a * y.b + x.b * y.d,
        x.c * y.a + x.d * y.c,
        x.c * y.b + x.d * y.d,
    )

impl Mul<Vec> for Mat2:
    type Out = Vec
    fn mul(lhs, rhs) -> Out = Vec(lhs.a * rhs.x + lhs.b * rhs.y, lhs.c * rhs.x + lhs.d * rhs.y)

impl Mul<F64> for Mat2:
    fn mul(lhs, k) = Mat2(lhs.a * k, lhs.b * k, lhs.c * k, lhs.d * k)

let u = Vec(1.0, 2.0)
let v = Vec(3.0, 4.0)
let s = (u + v) * 2.0
let d = s / 4.0
print(round(d.x), round(d.y))

let m = Mat2(2.0, 1.0, 0.0, 3.0)
let mv = m * u
print(round(mv.x), round(mv.y))

let m2 = m * m
let m2v = m2 * u
print(round(m2v.x), round(m2v.y))

let ms = m * 2.0
let msv = ms * u
print(round(msv.x), round(msv.y))

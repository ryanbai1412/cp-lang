// expect: ok, output "ok"
// input: "1 300\n"
// model: sin
// model: cos
// model: tan
// model: asin
// model: acos
// model: atan2
// model: hypot
// model: ln
// std.math's float fns against brute forces: sin, cos and exp as Taylor sums, tan as their
// ratio, the inverses by round trips through them, hypot by its square. Each x is k/1000 in
// [-3, 3]. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}

let pi = 3.141592653589793
let eps = 0.000000001

fn close(a: F64, b: F64) -> Bool = abs(a - b) <= eps * (1.0 + abs(b))

fn taylor(x: F64, odd: Bool) -> F64:
    var term = if odd: x else: 1.0
    var sum = 0.0
    var k = if odd: 1 else: 0
    for _ in 0..40:
        sum += term
        term = -term * x * x / F64((k + 1) * (k + 2))
        k += 2
    sum

fn texp(x: F64) -> F64:
    var term = 1.0
    var sum = 0.0
    for k in 1..80:
        sum += term
        term = term * x / F64(k)
    sum

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for op in 0..n:
    let x = F64(rng.int(-3000, 3000)) / 1000.0
    let (s, c) = (taylor(x, true), taylor(x, false))
    let half = x / 2.0
    let a = F64(rng.int(-1000, 1000))
    let b = F64(rng.int(-1000, 1000))
    if not close(sin(x), s): bad = "op {op}: sin({x})"
    elif not close(cos(x), c): bad = "op {op}: cos({x})"
    elif abs(c) > 0.01 and not close(tan(x), s / c): bad = "op {op}: tan({x})"
    elif not close(asin(sin(half)), half): bad = "op {op}: asin(sin({half}))"
    elif not close(acos(cos(abs(x))), abs(x)): bad = "op {op}: acos(cos({x}))"
    elif not close(atan2(s, c), x): bad = "op {op}: atan2({s}, {c})"
    elif not close(atan2(3.0 * s, 3.0 * c), x): bad = "op {op}: atan2 scaled at {x}"
    elif not close(hypot(a, b) * hypot(a, b), a * a + b * b): bad = "op {op}: hypot({a}, {b})"
    elif not close(exp(x), texp(x)): bad = "op {op}: exp({x})"
    elif not close(ln(texp(x)), x): bad = "op {op}: ln(exp({x}))"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)

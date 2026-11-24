// expect: ok, output "ok"
// input: "1 300\n"
// model: Exp
// model: pow_exp
// model: pow_tower
// Towers reduced down the totient chain against pow_mod with the exact exponent: seeded random
// moduli, bases and small towers, coprime or not. Reads `seed n`; prints the first op that differs.
use std.math.{phi, pow_mod, Exp, pow_exp, pow_tower}
use std.rand.{Rng}

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for op in 0..n:
    if bad != "": break
    let m = rng.int(1, 240)
    let (a, b, c, d) = (rng.int(0, 20), rng.int(0, 7), rng.int(0, 3), rng.int(0, 2))
    let e = rng.int(0, 500)
    let x = Exp.of(e, m)
    if x.v != (if e < m: e else: e % m + m) or x.m != m: bad = "op {op}: Exp.of"
    elif pow_exp(a, Exp.of(e, phi(m)), m).v % m != pow_mod(a, e, m): bad = "op {op}: pow_exp"
    elif pow_tower([a, b, c], m) != pow_mod(a, b ** c, m): bad = "op {op}: pow_tower"
    elif c ** d <= 3 and pow_tower([a, b, c, d], m) != pow_mod(a, b ** (c ** d), m):
        bad = "op {op}: pow_tower 4"
    elif pow_tower([a], m) != a % m or pow_tower([a, e], m) != pow_mod(a, e, m):
        bad = "op {op}: pow_tower short"
print(if bad == "": "ok" else: bad)

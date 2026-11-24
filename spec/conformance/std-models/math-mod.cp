// expect: ok, output "ok"
// input: "1 300\n"
// model: inv_mod
// model: crt
// model: crt_all
// model: mul_mod
// model: pow_mod
// model: mod_log
// model: mod_sqrt
// model: mod_kth_root
// model: linear_diophantine
// model: linear_congruence
// model: min_mod_in_range
// model: floor_sum
// std.math's modular arithmetic against search loops over every residue: seeded random small
// moduli (and huge ones for mul_mod and pow_mod), each answer compared with the least solution
// a loop finds. Reads `seed n`; prints the first op that differs.
use std.math.{inv_mod, crt, crt_all, mul_mod, pow_mod, mod_log, mod_sqrt, mod_kth_root}
use std.math.{linear_diophantine, linear_congruence, min_mod_in_range, floor_sum, is_prime}
use std.rand.{Rng}

fn md(a: Int, m: Int) -> Int = ((a % m) + m) % m

// a * b mod m by doubling, for 1 <= m < 2^62.
fn slow_mul(a: Int, b: Int, m: Int) -> Int:
    var (x, y, r) = (md(a, m), md(b, m), 0)
    while y > 0:
        if y % 2 == 1: r = (r + x) % m
        x = (x + x) % m
        y /= 2
    r

fn slow_pow(a: Int, e: Int, m: Int) -> Int:
    var r = 1 % m
    for _ in 0..e: r = r * md(a, m) % m
    r

fn first(m: Int, ok: Int -> Bool) -> ?Int:
    for x in 0..m:
        if ok(x): return Some(x)
    None

fn crt_brute(eqs: List<(Int, Int)>) -> ?(Int, Int):
    var l = 1
    for (_, m) in eqs: l = lcm(l, m)
    let r = first(l, x => all(md(x - r, m) == 0 for (r, m) in eqs))
    if r == None: None else: Some((r!, l))

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 11)
    let m = rng.int(1, 60)
    let a = rng.int(-100, 100)
    let b = rng.int(-100, 100)
    var p = rng.int(2, 200)
    while not is_prime(p): p += 1
    if op == 0:
        if inv_mod(a, m) != first(m, x => md(a * x, m) == 1 % m): bad = "op {k}: inv_mod({a}, {m})"
    elif op == 1:
        let m2 = rng.int(1, 60)
        if crt(a, m, b, m2) != crt_brute([(a, m), (b, m2)]): bad = "op {k}: crt({a}, {m}, {b}, {m2})"
    elif op == 2:
        let eqs = [(rng.int(-50, 50), rng.int(1, 12)) for _ in 0..rng.int(0, 4)]
        if crt_all(eqs) != crt_brute(eqs): bad = "op {k}: crt_all({eqs})"
    elif op == 3:
        let big = rng.int(1, 1 << 62)
        let (x, y) = (rng.int(Int.MIN + 1, Int.MAX), rng.int(Int.MIN + 1, Int.MAX))
        if mul_mod(x, y, big) != slow_mul(x, y, big) or mul_mod(a, b, m) != md(a * b, m):
            bad = "op {k}: mul_mod({x}, {y}, {big})"
    elif op == 4:
        let e = rng.int(0, 300)
        let big = rng.int(1, 1 << 62)
        var r = 1 % big
        let x = rng.int(Int.MIN + 1, Int.MAX)
        for _ in 0..e % 40: r = slow_mul(r, x, big)
        if pow_mod(a, e, m) != slow_pow(a, e, m) or pow_mod(x, e % 40, big) != r:
            bad = "op {k}: pow_mod({a}, {e}, {m})"
    elif op == 5:
        let want = first(2 * m + 2, x => slow_pow(a, x, m) == md(b, m))
        if mod_log(a, b, m) != want: bad = "op {k}: mod_log({a}, {b}, {m})"
    elif op == 6:
        if mod_sqrt(a, p) != first(p, x => x * x % p == md(a, p)): bad = "op {k}: mod_sqrt({a}, {p})"
    elif op == 7:
        let e = rng.int(1, 12)
        if mod_kth_root(a, e, p) != first(p, x => slow_pow(x, e, p) == md(a, p)): bad = "op {k}: mod_kth_root({a}, {e}, {p})"
    elif op == 8 and (a != 0 or b != 0):
        let c = rng.int(-300, 300)
        let g = gcd(a, b)
        let got = linear_diophantine(a, b, c)
        if c % g != 0:
            if got != None: bad = "op {k}: linear_diophantine({a}, {b}, {c}) has no solution"
        elif got == None: bad = "op {k}: linear_diophantine({a}, {b}, {c}) missed"
        else:
            let s = got
            let canon = if b != 0: 0 <= s.x0 < abs(b / g) else: s.x0 == c / a and s.y0 == 0
            if a * s.x0 + b * s.y0 != c or s.step_x != b / g or s.step_y != a / g or not canon:
                bad = "op {k}: linear_diophantine({a}, {b}, {c}) = {s}"
    elif op == 9:
        let sols = [x for x in 0..m if md(a * x - b, m) == 0]
        let want = if sols.len == 0: None else: Some((sols[0], m / sols.len))
        let ok = sols.len == 0 or all(md(sols[i] - sols[0], m / sols.len) == 0 for i in 0..sols.len)
        if not ok or linear_congruence(a, b, m) != want: bad = "op {k}: linear_congruence({a}, {b}, {m})"
    elif op == 10:
        let lo = rng.int(0, m - 1)
        let hi = rng.int(lo, m - 1)
        let want = first(m, x => lo <= md(a * x, m) <= hi)
        if min_mod_in_range(a, m, lo, hi) != want: bad = "op {k}: min_mod_in_range({a}, {m}, {lo}, {hi})"
    elif op == 11:
        let cnt = rng.int(0, 50)
        if floor_sum(cnt, m, a, b) != sum((a * i + b) / m for i in 0..cnt): bad = "op {k}: floor_sum({cnt}, {m}, {a}, {b})"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
